use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant, SystemTime};

use notify::{RecursiveMode, Watcher};
use serde_json::Value;

use super::sessions::POLL;
use crate::source::SourceEvent;
use crate::state::Event;

pub const FIRST_SCAN_CAP: u64 = 256 * 1024;
const DEBOUNCE: Duration = Duration::from_millis(50);
const ERROR_SUMMARY_MAX: usize = 80;

/// Project directory name under `~/.claude/projects`: every char that is not ASCII alphanumeric becomes `-`.
pub fn slug(cwd: &str) -> String {
    cwd.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect()
}

fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Tries the slug directory first, then any project directory (long cwds get a truncated, hashed slug).
pub fn locate(projects: &Path, cwd: &str, session_id: &str) -> Option<PathBuf> {
    if !safe_id(session_id) {
        return None;
    }
    let file = format!("{session_id}.jsonl");
    if !cwd.is_empty() {
        let direct = projects.join(slug(cwd)).join(&file);
        if direct.is_file() {
            return Some(direct);
        }
    }
    std::fs::read_dir(projects)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path().join(&file))
        .find(|p| p.is_file())
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Model(String),
    ToolUse { id: String, name: String, target: Option<String> },
    ToolResult { id: String },
    ApiError { summary: String },
}

/// Parses one transcript line. Sidechain (subagent) lines and unrelated line types yield nothing.
pub fn parse_line(line: &str, cwd: &str) -> Result<Vec<Item>, String> {
    let v: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    let obj = v.as_object().ok_or("not an object")?;
    if obj.get("isSidechain").and_then(Value::as_bool) == Some(true) {
        return Ok(Vec::new());
    }
    let blocks = v.pointer("/message/content").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    let str_of = |b: &Value, k: &str| b.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    let mut items = Vec::new();
    match obj.get("type").and_then(Value::as_str) {
        Some("assistant") => {
            if obj.get("isApiErrorMessage").and_then(Value::as_bool) == Some(true) {
                return Ok(vec![Item::ApiError { summary: error_summary(&v, blocks) }]);
            }
            if let Some(m) = v.pointer("/message/model").and_then(Value::as_str) {
                if !m.is_empty() && !m.starts_with('<') {
                    items.push(Item::Model(pretty_model(m)));
                }
            }
            for b in blocks.iter().filter(|b| block_type(b) == "tool_use") {
                let name = str_of(b, "name");
                if name.is_empty() {
                    continue;
                }
                let target = b.get("input").and_then(|i| tool_target(i, cwd));
                items.push(Item::ToolUse { id: str_of(b, "id"), name, target });
            }
        }
        Some("user") => {
            for b in blocks.iter().filter(|b| block_type(b) == "tool_result") {
                items.push(Item::ToolResult { id: str_of(b, "tool_use_id") });
            }
        }
        _ => {}
    }
    Ok(items)
}

fn block_type(b: &Value) -> &str {
    b.get("type").and_then(Value::as_str).unwrap_or_default()
}

fn error_summary(v: &Value, blocks: &[Value]) -> String {
    let text = blocks
        .iter()
        .filter(|b| block_type(b) == "text")
        .filter_map(|b| b.get("text").and_then(Value::as_str))
        .flat_map(str::lines)
        .map(str::trim)
        .find(|l| !l.is_empty());
    let summary = match text {
        Some(t) => t.to_string(),
        None => {
            let code = v.get("error").and_then(Value::as_str).unwrap_or("unknown");
            match v.get("apiErrorStatus").and_then(Value::as_u64) {
                Some(status) => format!("API error: {code} ({status})"),
                None => format!("API error: {code}"),
            }
        }
    };
    truncate(&summary, ERROR_SUMMARY_MAX)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

/// `claude-opus-5-5` → `Opus 5.5`, `claude-3-5-sonnet-20241022` → `Sonnet 3.5`. Unknown shapes pass through.
pub fn pretty_model(id: &str) -> String {
    let Some(rest) = id.strip_prefix("claude-") else {
        return id.to_string();
    };
    let rest = rest.split('[').next().unwrap_or(rest);
    let mut family = Vec::new();
    let mut version = Vec::new();
    for part in rest.split('-').filter(|p| !p.is_empty()) {
        if part.chars().all(|c| c.is_ascii_digit()) {
            if part.len() <= 2 {
                version.push(part);
            }
        } else {
            let mut c = part.chars();
            family.push(c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default());
        }
    }
    if family.is_empty() {
        return id.to_string();
    }
    let mut out = family.join(" ");
    if !version.is_empty() {
        out.push(' ');
        out.push_str(&version.join("."));
    }
    out
}

fn tool_target(input: &Value, cwd: &str) -> Option<String> {
    let s = |k: &str| input.get(k).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty());
    if let Some(p) = s("file_path").or_else(|| s("notebook_path")) {
        return Some(relative(p, cwd));
    }
    if let Some(c) = s("command") {
        return c.lines().next().map(|l| l.trim().to_string());
    }
    ["pattern", "description", "url", "query", "skill", "path"]
        .iter()
        .find_map(|k| s(k))
        .map(str::to_string)
}

/// Path relative to `cwd` with forward slashes when inside it, else unchanged.
fn relative(path: &str, cwd: &str) -> String {
    let p = path.replace('\\', "/");
    let base = cwd.replace('\\', "/");
    let base = base.trim_end_matches('/');
    if !base.is_empty() && p.len() > base.len() + 1 {
        let (head, tail) = p.split_at(base.len());
        if head.eq_ignore_ascii_case(base) && tail.starts_with('/') {
            return tail[1..].to_string();
        }
    }
    path.to_string()
}

/// Byte-offset reader for one append-only JSONL file.
pub struct Tail {
    path: PathBuf,
    offset: u64,
    created: Option<SystemTime>,
    partial: Vec<u8>,
    skip_to_newline: bool,
    primed: bool,
}

impl Tail {
    pub fn new(path: PathBuf) -> Self {
        Tail { path, offset: 0, created: None, partial: Vec::new(), skip_to_newline: false, primed: false }
    }

    /// Complete lines appended since the last read; `true` when this read (re)started from the last `FIRST_SCAN_CAP` bytes.
    pub fn read(&mut self) -> std::io::Result<(Vec<String>, bool)> {
        let mut f = File::open(&self.path)?;
        let meta = f.metadata()?;
        let len = meta.len();
        let created = meta.created().ok();
        if self.primed && (len < self.offset || created != self.created) {
            self.primed = false;
        }
        let first = !self.primed;
        if first {
            let start = len.saturating_sub(FIRST_SCAN_CAP);
            // Start one byte early so a line beginning exactly at the cap is kept: everything through the first `\n` is dropped.
            self.offset = start.saturating_sub(1);
            self.skip_to_newline = start > 0;
            self.partial.clear();
            self.created = created;
            self.primed = true;
        }
        if len <= self.offset {
            return Ok((Vec::new(), first));
        }
        f.seek(SeekFrom::Start(self.offset))?;
        let mut buf = Vec::new();
        f.take(len - self.offset).read_to_end(&mut buf)?;
        self.offset += buf.len() as u64;

        let mut data = std::mem::take(&mut self.partial);
        data.extend_from_slice(&buf);
        if self.skip_to_newline {
            match data.iter().position(|&b| b == b'\n') {
                Some(i) => {
                    data.drain(..=i);
                    self.skip_to_newline = false;
                }
                None => return Ok((Vec::new(), first)),
            }
        }
        let cut = data.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        self.partial = data.split_off(cut);
        let lines = data
            .split(|&b| b == b'\n')
            .map(|l| String::from_utf8_lossy(l).trim_end_matches('\r').to_string())
            .filter(|l| !l.trim().is_empty())
            .collect();
        Ok((lines, first))
    }
}

/// Turns parsed items into state-engine events. A first scan replays history, so it only reports the model and a still-running tool.
pub fn translate(items: Vec<Item>, first: bool, last_model: &mut Option<String>) -> Vec<Event> {
    let mut out = Vec::new();
    let mut model_event = |m: String, out: &mut Vec<Event>| {
        if last_model.as_deref() != Some(m.as_str()) {
            *last_model = Some(m.clone());
            out.push(Event::TranscriptModel { model: m });
        }
    };
    if first {
        let mut pending: Vec<(String, String, Option<String>)> = Vec::new();
        let mut model = None;
        for item in items {
            match item {
                Item::Model(m) => model = Some(m),
                Item::ToolUse { id, name, target } => pending.push((id, name, target)),
                Item::ToolResult { id } => pending.retain(|(p, _, _)| *p != id),
                Item::ApiError { .. } => pending.clear(),
            }
        }
        if let Some(m) = model {
            model_event(m, &mut out);
        }
        if let Some((_, tool, target)) = pending.pop() {
            out.push(Event::TranscriptToolUse { tool, target });
        }
        return out;
    }
    for item in items {
        match item {
            Item::Model(m) => model_event(m, &mut out),
            Item::ToolUse { name, target, .. } => out.push(Event::TranscriptToolUse { tool: name, target }),
            Item::ToolResult { .. } => out.push(Event::TranscriptToolResult),
            Item::ApiError { summary } => out.push(Event::TranscriptApiError { summary }),
        }
    }
    out
}

pub enum Msg {
    Track { id: String, cwd: String },
    Untrack { id: String },
    Fs(notify::Result<notify::Event>),
}

struct Tracked {
    cwd: String,
    tail: Option<Tail>,
    last_model: Option<String>,
}

/// Tails the transcripts of the sessions it is told about.
pub struct Tailer {
    projects: PathBuf,
    sessions: HashMap<String, Tracked>,
    warned: HashSet<String>,
}

impl Tailer {
    pub fn new(projects: PathBuf) -> Self {
        Tailer { projects, sessions: HashMap::new(), warned: HashSet::new() }
    }

    pub fn track(&mut self, id: &str, cwd: &str) -> Vec<SourceEvent> {
        match self.sessions.get(id) {
            Some(t) if t.cwd == cwd => return Vec::new(),
            _ => {}
        }
        self.sessions.insert(id.to_string(), Tracked { cwd: cwd.to_string(), tail: None, last_model: None });
        self.read(id)
    }

    pub fn untrack(&mut self, id: &str) {
        self.sessions.remove(id);
        self.warned.remove(id);
    }

    /// Reads every tracked transcript, locating any not found yet.
    pub fn poll(&mut self) -> Vec<SourceEvent> {
        let mut ids: Vec<String> = self.sessions.keys().cloned().collect();
        ids.sort();
        ids.iter().flat_map(|id| self.read(id)).collect()
    }

    /// Reads the tracked transcripts named in a filesystem event.
    pub fn changed(&mut self, paths: &[PathBuf]) -> Vec<SourceEvent> {
        let mut ids: Vec<String> = paths
            .iter()
            .filter_map(|p| p.file_name()?.to_str()?.strip_suffix(".jsonl").map(str::to_string))
            .filter(|id| self.sessions.contains_key(id))
            .collect();
        ids.sort();
        ids.dedup();
        ids.iter().flat_map(|id| self.read(id)).collect()
    }

    fn read(&mut self, id: &str) -> Vec<SourceEvent> {
        let projects = &self.projects;
        let Some(t) = self.sessions.get_mut(id) else { return Vec::new() };
        if t.tail.is_none() {
            t.tail = locate(projects, &t.cwd, id).map(Tail::new);
        }
        let Some(tail) = t.tail.as_mut() else { return Vec::new() };
        let (lines, first) = match tail.read() {
            Ok(r) => r,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    t.tail = None;
                } else if self.warned.insert(id.to_string()) {
                    eprintln!("peepthis: cannot read transcript {}: {e}", tail.path.display());
                }
                return Vec::new();
            }
        };
        let mut items = Vec::new();
        let mut bad = 0;
        for line in &lines {
            match parse_line(line, &t.cwd) {
                Ok(mut it) => items.append(&mut it),
                Err(_) => bad += 1,
            }
        }
        if bad > 0 {
            eprintln!("peepthis: skipped {bad} malformed transcript line(s) in {}", tail.path.display());
        }
        translate(items, first, &mut t.last_model)
            .into_iter()
            .map(|event| SourceEvent::Apply { id: id.to_string(), event })
            .collect()
    }
}

/// Runs forever: watches `projects` recursively, reads on change and every `POLL`.
pub fn run(projects: PathBuf, tx: Sender<Msg>, rx: Receiver<Msg>, out: impl Fn(SourceEvent)) {
    let mut tailer = Tailer::new(projects.clone());
    let mut watcher = match notify::recommended_watcher(move |res| {
        let _ = tx.send(Msg::Fs(res));
    }) {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!("peepthis: transcript watcher unavailable, polling only: {e}");
            None
        }
    };
    let mut watching = false;
    let mut next_poll = Instant::now() + POLL;

    loop {
        if let Some(w) = watcher.as_mut().filter(|_| !watching && projects.is_dir()) {
            watching = w.watch(&projects, RecursiveMode::Recursive).is_ok();
        }
        let first = match rx.recv_timeout(next_poll.saturating_duration_since(Instant::now())) {
            Ok(msg) => msg,
            Err(RecvTimeoutError::Timeout) => {
                tailer.poll().into_iter().for_each(&out);
                next_poll = Instant::now() + POLL;
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => return,
        };
        let mut msgs = vec![first];
        if matches!(msgs[0], Msg::Fs(_)) {
            std::thread::sleep(DEBOUNCE);
        }
        msgs.extend(rx.try_iter());

        let mut paths = Vec::new();
        let mut failed = false;
        for msg in msgs {
            match msg {
                Msg::Track { id, cwd } => tailer.track(&id, &cwd).into_iter().for_each(&out),
                Msg::Untrack { id } => tailer.untrack(&id),
                Msg::Fs(Ok(ev)) => paths.extend(ev.paths),
                Msg::Fs(Err(_)) => failed = true,
            }
        }
        if failed {
            if let Some(w) = watcher.as_mut() {
                let _ = w.unwatch(&projects);
            }
            watching = false;
            tailer.poll().into_iter().for_each(&out);
        } else if !paths.is_empty() {
            tailer.changed(&paths).into_iter().for_each(&out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    const CWD: &str = r"C:\work\demo";
    const SID: &str = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

    fn fixture(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/transcripts").join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn temp_dir() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "peepthis-transcript-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn parse_all(text: &str) -> (Vec<Item>, usize) {
        let mut items = Vec::new();
        let mut bad = 0;
        for line in text.lines() {
            match parse_line(line, CWD) {
                Ok(mut it) => items.append(&mut it),
                Err(_) => bad += 1,
            }
        }
        (items, bad)
    }

    fn append(path: &Path, text: &str) {
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
        f.write_all(text.as_bytes()).unwrap();
    }

    fn events(evs: Vec<SourceEvent>) -> Vec<Event> {
        evs.into_iter()
            .map(|e| match e {
                SourceEvent::Apply { id, event } => {
                    assert_eq!(id, SID);
                    event
                }
                other => panic!("unexpected {other:?}"),
            })
            .collect()
    }

    #[test]
    fn transcript_slug_matches_observed_directory_names() {
        assert_eq!(slug(r"F:\Projects\agentview widget"), "F--Projects-agentview-widget");
        assert_eq!(slug(r"C:\Users\dev"), "C--Users-dev");
        assert_eq!(slug(r"C:\a\datamap.cymru"), "C--a-datamap-cymru");
        assert_eq!(slug("/home/u/my_app"), "-home-u-my-app");
    }

    #[test]
    fn transcript_locate_prefers_slug_dir_then_scans() {
        let root = temp_dir();
        let dir = root.join(slug(CWD));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{SID}.jsonl")), "").unwrap();
        assert_eq!(locate(&root, CWD, SID), Some(dir.join(format!("{SID}.jsonl"))));

        let other = root.join("C--hashed-long-slug-1a2b");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("bbbb.jsonl"), "").unwrap();
        assert_eq!(locate(&root, r"C:\elsewhere", "bbbb"), Some(other.join("bbbb.jsonl")));
        assert_eq!(locate(&root, CWD, "missing"), None);
        assert_eq!(locate(&root, CWD, "..\\x"), None);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transcript_parses_model_tools_and_results() {
        let (items, bad) = parse_all(&fixture("basic.jsonl"));
        assert_eq!(bad, 0);
        assert_eq!(
            items,
            vec![
                Item::Model("Opus 5.5".into()),
                Item::Model("Opus 5.5".into()),
                Item::ToolUse {
                    id: "toolu_placeholder_1".into(),
                    name: "Edit".into(),
                    target: Some("src/main.ts".into())
                },
                Item::ToolResult { id: "toolu_placeholder_1".into() },
                Item::Model("Opus 5.5".into()),
                Item::ToolUse { id: "toolu_placeholder_2".into(), name: "Bash".into(), target: Some("npm test".into()) },
                Item::ToolResult { id: "toolu_placeholder_2".into() },
                Item::Model("Opus 5.5".into()),
            ]
        );
    }

    #[test]
    fn transcript_api_error_marker() {
        let (items, bad) = parse_all(&fixture("api_error.jsonl"));
        assert_eq!(bad, 0);
        assert_eq!(
            items,
            vec![
                Item::ApiError { summary: "You've hit your 5-hour limit · resets 8:00pm".into() },
                Item::ApiError { summary: "API error: server_error".into() },
            ]
        );
    }

    #[test]
    fn transcript_skips_malformed_lines() {
        let (items, bad) = parse_all(&fixture("malformed.jsonl"));
        assert_eq!(bad, 3);
        assert_eq!(
            items,
            vec![
                Item::Model("Sonnet 4.5".into()),
                Item::ToolUse { id: "toolu_placeholder_1".into(), name: "Grep".into(), target: Some("TODO".into()) },
                Item::ToolResult { id: "toolu_placeholder_1".into() },
            ]
        );
    }

    #[test]
    fn transcript_pretty_model_names() {
        assert_eq!(pretty_model("claude-opus-5-5"), "Opus 5.5");
        assert_eq!(pretty_model("claude-sonnet-4-5-20250929"), "Sonnet 4.5");
        assert_eq!(pretty_model("claude-3-5-sonnet-20241022"), "Sonnet 3.5");
        assert_eq!(pretty_model("claude-opus-4-7[1m]"), "Opus 4.7");
        assert_eq!(pretty_model("claude-fable"), "Fable");
        assert_eq!(pretty_model("gpt-x"), "gpt-x");
    }

    #[test]
    fn transcript_targets() {
        let t = |v: Value| tool_target(&v, CWD);
        assert_eq!(t(serde_json::json!({"file_path": r"C:\work\demo\a\b.rs"})), Some("a/b.rs".into()));
        assert_eq!(t(serde_json::json!({"file_path": r"c:/WORK/demo/x.ts"})), Some("x.ts".into()));
        assert_eq!(t(serde_json::json!({"file_path": r"D:\other\x.ts"})), Some(r"D:\other\x.ts".into()));
        assert_eq!(t(serde_json::json!({"file_path": r"C:\work\demolition\x"})), Some(r"C:\work\demolition\x".into()));
        assert_eq!(t(serde_json::json!({"pattern": "foo", "path": "src"})), Some("foo".into()));
        assert_eq!(t(serde_json::json!({"url": "https://example.com"})), Some("https://example.com".into()));
        assert_eq!(t(serde_json::json!({"todos": []})), None);
    }

    #[test]
    fn transcript_first_scan_reports_model_and_running_tool_only() {
        let mut model = None;
        let (items, _) = parse_all(&fixture("pending_tool.jsonl"));
        assert_eq!(
            translate(items, true, &mut model),
            vec![
                Event::TranscriptModel { model: "Opus 5.5".into() },
                Event::TranscriptToolUse { tool: "Agent".into(), target: Some("<description placeholder>".into()) },
            ]
        );
        let (items, _) = parse_all(&fixture("basic.jsonl"));
        assert_eq!(translate(items, true, &mut model), vec![], "finished tools and a known model emit nothing");
        let (items, _) = parse_all(&fixture("api_error.jsonl"));
        assert_eq!(translate(items, true, &mut None), vec![], "old API errors are not replayed");
    }

    #[test]
    fn transcript_live_reads_emit_every_event_in_order() {
        let mut model = Some("Opus 5.5".to_string());
        let (items, _) = parse_all(&fixture("basic.jsonl"));
        assert_eq!(
            translate(items, false, &mut model),
            vec![
                Event::TranscriptToolUse { tool: "Edit".into(), target: Some("src/main.ts".into()) },
                Event::TranscriptToolResult,
                Event::TranscriptToolUse { tool: "Bash".into(), target: Some("npm test".into()) },
                Event::TranscriptToolResult,
            ]
        );
        let (items, _) = parse_all(&fixture("api_error.jsonl"));
        let evs = translate(items, false, &mut model);
        assert!(matches!(&evs[0], Event::TranscriptApiError { summary } if summary.starts_with("You've hit")));
        assert_eq!(evs.len(), 2);
    }

    #[test]
    fn transcript_tail_reads_offsets_and_holds_partial_lines() {
        let dir = temp_dir();
        let path = dir.join("t.jsonl");
        append(&path, "one\ntwo\nthr");
        let mut tail = Tail::new(path.clone());
        assert_eq!(tail.read().unwrap(), (vec!["one".into(), "two".into()], true));
        assert_eq!(tail.read().unwrap(), (vec![], false));
        append(&path, "ee\r\nfour\n");
        assert_eq!(tail.read().unwrap(), (vec!["three".into(), "four".into()], false));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn transcript_tail_first_scan_capped_and_skips_partial_line() {
        let dir = temp_dir();
        let path = dir.join("t.jsonl");
        let filler = format!("{}\n", "x".repeat(1023));
        let mut body = String::from("head-line-cut-by-cap\n");
        while body.len() < (FIRST_SCAN_CAP as usize) + 4096 {
            body.push_str(&filler);
        }
        body.push_str("last\n");
        append(&path, &body);
        let mut tail = Tail::new(path.clone());
        let (lines, first) = tail.read().unwrap();
        assert!(first);
        assert_eq!(lines.last().map(String::as_str), Some("last"));
        assert!(lines.iter().all(|l| l.len() == 1023 || l == "last"), "partial first line must be dropped");
        let read: usize = lines.iter().map(|l| l.len() + 1).sum();
        assert!(read as u64 <= FIRST_SCAN_CAP);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn transcript_tail_keeps_line_starting_exactly_at_cap() {
        let dir = temp_dir();
        let path = dir.join("t.jsonl");
        let line = format!("{}\n", "y".repeat(1023));
        let mut body = "z".repeat(99) + "\n";
        for _ in 0..(FIRST_SCAN_CAP / 1024) {
            body.push_str(&line);
        }
        append(&path, &body);
        let (lines, _) = Tail::new(path).read().unwrap();
        assert_eq!(lines.len() as u64, FIRST_SCAN_CAP / 1024);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn transcript_tail_resets_on_truncation() {
        let dir = temp_dir();
        let path = dir.join("t.jsonl");
        append(&path, "aaaa\nbbbb\n");
        let mut tail = Tail::new(path.clone());
        tail.read().unwrap();
        std::fs::write(&path, "c\n").unwrap();
        assert_eq!(tail.read().unwrap(), (vec!["c".into()], true));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn transcript_tailer_tracks_locates_late_and_reads_appends() {
        let root = temp_dir();
        let mut tailer = Tailer::new(root.clone());
        assert!(tailer.track(SID, CWD).is_empty(), "no transcript yet");
        assert!(tailer.track(SID, CWD).is_empty(), "re-track is a no-op");

        let dir = root.join(slug(CWD));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{SID}.jsonl"));
        append(&path, &fixture("pending_tool.jsonl"));
        assert_eq!(
            events(tailer.poll()),
            vec![
                Event::TranscriptModel { model: "Opus 5.5".into() },
                Event::TranscriptToolUse { tool: "Agent".into(), target: Some("<description placeholder>".into()) },
            ]
        );

        append(&path, &fixture("malformed.jsonl"));
        assert_eq!(
            events(tailer.changed(&[path.clone(), dir.join("unrelated.jsonl")])),
            vec![
                Event::TranscriptModel { model: "Sonnet 4.5".into() },
                Event::TranscriptToolUse { tool: "Grep".into(), target: Some("TODO".into()) },
                Event::TranscriptToolResult,
            ]
        );
        assert!(tailer.poll().is_empty());

        tailer.untrack(SID);
        append(&path, &fixture("api_error.jsonl"));
        assert!(tailer.poll().is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transcript_run_thread_emits_for_tracked_sessions() {
        let root = temp_dir();
        let dir = root.join(slug(CWD));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{SID}.jsonl"));
        append(&path, &fixture("pending_tool.jsonl"));

        let (tx, rx) = std::sync::mpsc::channel();
        let (out_tx, out_rx) = std::sync::mpsc::channel();
        let ctl = tx.clone();
        let projects = root.clone();
        std::thread::spawn(move || run(projects, tx, rx, move |ev| {
            let _ = out_tx.send(ev);
        }));
        ctl.send(Msg::Track { id: SID.into(), cwd: CWD.into() }).unwrap();
        let first = out_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(first, SourceEvent::Apply { event: Event::TranscriptModel { .. }, .. }));
        out_rx.recv_timeout(Duration::from_secs(5)).unwrap();

        append(&path, &fixture("api_error.jsonl"));
        let ev = out_rx.recv_timeout(POLL + Duration::from_secs(2)).unwrap();
        assert!(matches!(ev, SourceEvent::Apply { event: Event::TranscriptApiError { .. }, .. }));
    }
}
