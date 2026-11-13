use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use serde::Deserialize;
use serde_json::Value;

use crate::source::process::{self, Proc};
use crate::source::{SessionInfo, Sink, SourceEvent};

pub const POLL: Duration = Duration::from_secs(5);
const DEBOUNCE: Duration = Duration::from_millis(50);
/// `procStart` and the OS creation time are the same FILETIME; allow 2s for clock rounding.
const PROC_START_TOLERANCE: u64 = 2 * 10_000_000;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Raw {
    pid: Option<u32>,
    session_id: Option<String>,
    cwd: Option<String>,
    proc_start: Option<Value>,
    kind: Option<String>,
    entrypoint: Option<String>,
    name: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub info: SessionInfo,
    pub busy: Option<bool>,
    pub proc_start: Option<u64>,
}

/// Parses one session file. `stem` is the file name without `.json`, used when `pid` is absent.
pub fn parse(bytes: &[u8], stem: &str) -> Result<Record, String> {
    let raw: Raw = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let id = raw
        .session_id
        .filter(|s| !s.trim().is_empty())
        .ok_or("missing sessionId")?;
    let pid = raw.pid.or_else(|| stem.parse().ok()).ok_or("missing pid")?;
    let cwd = raw.cwd.unwrap_or_default();
    let name = raw
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| fallback_name(&cwd));
    let busy = match raw.status.as_deref() {
        Some("busy") => Some(true),
        Some("idle") => Some(false),
        _ => None,
    };
    let proc_start = match raw.proc_start {
        Some(Value::String(s)) => s.trim().parse().ok(),
        Some(Value::Number(n)) => n.as_u64(),
        _ => None,
    };
    Ok(Record {
        info: SessionInfo {
            id,
            pid,
            name,
            cwd,
            kind: raw.kind.unwrap_or_default(),
            entrypoint: raw.entrypoint.unwrap_or_default(),
        },
        busy,
        proc_start,
    })
}

fn fallback_name(cwd: &str) -> String {
    cwd.trim_end_matches(['\\', '/'])
        .rsplit(['\\', '/'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("claude")
        .to_string()
}

/// A session is live when its PID runs and, if both are known, the process start matches `procStart` (guards PID reuse).
pub fn is_live(rec: &Record, probe: impl Fn(u32) -> Proc) -> bool {
    match probe(rec.info.pid) {
        Proc::Dead => false,
        Proc::Alive { started: Some(t) } => {
            rec.proc_start.is_none_or(|ps| t.abs_diff(ps) <= PROC_START_TOLERANCE)
        }
        Proc::Alive { started: None } => true,
    }
}

pub struct SessionDir {
    dir: PathBuf,
    /// Live session id per file, from the last scan.
    known: HashMap<PathBuf, String>,
    warned: HashSet<PathBuf>,
}

impl SessionDir {
    pub fn new(dir: PathBuf) -> Self {
        SessionDir { dir, known: HashMap::new(), warned: HashSet::new() }
    }

    pub fn scan(&mut self) -> Vec<SourceEvent> {
        self.scan_with(process::probe)
    }

    /// Full rescan. Sessions dead on first sight are ignored; a known session ends when its process dies or its file goes.
    pub fn scan_with(&mut self, probe: impl Fn(u32) -> Proc) -> Vec<SourceEvent> {
        let mut events = Vec::new();
        let mut live: HashMap<PathBuf, String> = HashMap::new();
        let mut ended: HashSet<String> = HashSet::new();

        for path in json_files(&self.dir) {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
            let parsed = std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| parse(&b, &stem));
            match parsed {
                Ok(rec) => {
                    self.warned.remove(&path);
                    if is_live(&rec, &probe) {
                        live.insert(path, rec.info.id.clone());
                        events.push(SourceEvent::Upsert { info: rec.info, busy: rec.busy });
                    } else if self.known.values().any(|id| *id == rec.info.id) {
                        ended.insert(rec.info.id);
                    }
                }
                Err(err) => {
                    if self.warned.insert(path.clone()) {
                        eprintln!("peepthis: skipping session file {}: {err}", path.display());
                    }
                    // A file caught mid-write keeps its previous session alive until the next scan.
                    if let Some(id) = self.known.get(&path) {
                        live.insert(path, id.clone());
                    }
                }
            }
        }

        let still_live: HashSet<&String> = live.values().collect();
        for id in self.known.values() {
            if !still_live.contains(id) {
                ended.insert(id.clone());
            }
        }
        let mut ended: Vec<String> = ended.into_iter().filter(|id| !still_live.contains(id)).collect();
        ended.sort();
        events.extend(ended.into_iter().map(|id| SourceEvent::Ended { id }));

        self.known = live;
        events
    }

    /// Rescans on every filesystem change to a `.json` file, and every `POLL` regardless.
    pub fn watch(mut self, sink: Sink) {
        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = match notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        }) {
            Ok(w) => Some(w),
            Err(e) => {
                eprintln!("peepthis: file watcher unavailable, polling only: {e}");
                None
            }
        };
        let mut watching = false;

        loop {
            if let Some(w) = watcher.as_mut().filter(|_| !watching && self.dir.is_dir()) {
                watching = w.watch(&self.dir, RecursiveMode::NonRecursive).is_ok();
            }
            match rx.recv_timeout(POLL) {
                Ok(first) => {
                    std::thread::sleep(DEBOUNCE);
                    let mut relevant = touches_json(&first);
                    let mut failed = first.is_err();
                    for res in rx.try_iter() {
                        relevant |= touches_json(&res);
                        failed |= res.is_err();
                    }
                    if failed {
                        if let Some(w) = watcher.as_mut() {
                            let _ = w.unwatch(&self.dir);
                        }
                        watching = false;
                    }
                    if !relevant && !failed {
                        continue;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => watcher = None,
            }
            for ev in self.scan() {
                sink(ev);
            }
        }
    }
}

fn is_json(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("json"))
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
        .filter(|p| is_json(p))
        .collect();
    files.sort();
    files
}

fn touches_json(res: &notify::Result<notify::Event>) -> bool {
    match res {
        Ok(ev) => ev.paths.is_empty() || ev.paths.iter().any(|p| is_json(p)),
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn fixture(name: &str) -> Vec<u8> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sessions").join(name);
        std::fs::read(path).unwrap()
    }

    fn temp_dir() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "peepthis-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn place(dir: &Path, fixture_name: &str, as_name: &str) {
        std::fs::write(dir.join(as_name), fixture(fixture_name)).unwrap();
    }

    const BUSY_START: u64 = 134390522597698711;

    fn alive_matching(_pid: u32) -> Proc {
        Proc::Alive { started: Some(BUSY_START) }
    }

    fn alive_unknown(_pid: u32) -> Proc {
        Proc::Alive { started: None }
    }

    fn dead(_pid: u32) -> Proc {
        Proc::Dead
    }

    #[test]
    fn parses_full_session_file() {
        let rec = parse(&fixture("busy.json"), "18504").unwrap();
        assert_eq!(rec.info.id, "11111111-2222-4333-8444-555555555555");
        assert_eq!(rec.info.pid, 18504);
        assert_eq!(rec.info.name, "demo-app-24");
        assert_eq!(rec.info.cwd, r"C:\Users\dev\projects\demo-app");
        assert_eq!(rec.info.kind, "interactive");
        assert_eq!(rec.info.entrypoint, "cli");
        assert_eq!(rec.busy, Some(true));
        assert_eq!(rec.proc_start, Some(BUSY_START));
    }

    #[test]
    fn maps_status_values() {
        assert_eq!(parse(&fixture("idle.json"), "x").unwrap().busy, Some(false));
        assert_eq!(parse(&fixture("shell.json"), "x").unwrap().busy, None);
        assert_eq!(parse(&fixture("minimal.json"), "x").unwrap().busy, None);
    }

    #[test]
    fn tolerates_missing_and_unknown_fields() {
        let rec = parse(&fixture("minimal.json"), "4242").unwrap();
        assert_eq!(rec.info.pid, 4242);
        assert_eq!(rec.info.name, "claude");
        assert_eq!(rec.info.cwd, "");
        assert_eq!(rec.info.kind, "");
        assert_eq!(rec.proc_start, None);
    }

    #[test]
    fn falls_back_to_cwd_basename_and_filename_pid() {
        let rec = parse(br#"{"sessionId":"s","cwd":"D:\\code\\thing\\","procStart":12345}"#, "777").unwrap();
        assert_eq!(rec.info.name, "thing");
        assert_eq!(rec.info.pid, 777);
        assert_eq!(rec.proc_start, Some(12345));
    }

    #[test]
    fn rejects_malformed_or_unidentifiable_files() {
        assert!(parse(&fixture("malformed.json"), "5150").is_err());
        assert!(parse(br#"{"pid":1}"#, "1").is_err());
        assert!(parse(br#"{"sessionId":"s"}"#, "notapid").is_err());
        assert!(parse(b"[]", "1").is_err());
    }

    #[test]
    fn liveness_checks_pid_and_proc_start() {
        let rec = parse(&fixture("busy.json"), "x").unwrap();
        assert!(is_live(&rec, alive_matching));
        assert!(is_live(&rec, |_| Proc::Alive { started: Some(BUSY_START + 10_000_000) }));
        assert!(!is_live(&rec, |_| Proc::Alive { started: Some(BUSY_START + 30_000_000) }));
        assert!(!is_live(&rec, |_| Proc::Alive { started: Some(BUSY_START - 30_000_000) }));
        assert!(is_live(&rec, alive_unknown));
        assert!(!is_live(&rec, dead));
        let no_start = parse(&fixture("minimal.json"), "x").unwrap();
        assert!(is_live(&no_start, |_| Proc::Alive { started: Some(1) }));
    }

    #[test]
    fn liveness_against_real_process() {
        let me = std::process::id();
        let mut rec = parse(&fixture("minimal.json"), "x").unwrap();
        rec.info.pid = me;
        assert!(is_live(&rec, process::probe));
        if cfg!(windows) {
            rec.proc_start = Some(1);
            assert!(!is_live(&rec, process::probe), "stale procStart should mean PID reuse");
            if let Proc::Alive { started } = process::probe(me) {
                rec.proc_start = started;
            }
            assert!(is_live(&rec, process::probe));
        }
    }

    #[test]
    fn scan_emits_live_sessions_and_skips_bad_and_non_json_files() {
        let dir = temp_dir();
        place(&dir, "busy.json", "18504.json");
        place(&dir, "malformed.json", "5150.json");
        std::fs::write(dir.join("18504.key"), "dummy").unwrap();
        std::fs::write(dir.join("notes.txt"), "{\"sessionId\":\"txt\",\"pid\":1}").unwrap();

        let mut sd = SessionDir::new(dir.clone());
        let events = sd.scan_with(alive_matching);
        assert_eq!(events.len(), 1);
        match &events[0] {
            SourceEvent::Upsert { info, busy } => {
                assert_eq!(info.name, "demo-app-24");
                assert_eq!(*busy, Some(true));
            }
            other => panic!("unexpected {other:?}"),
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn dead_on_first_sight_is_ignored() {
        let dir = temp_dir();
        place(&dir, "busy.json", "18504.json");
        let mut sd = SessionDir::new(dir.clone());
        assert!(sd.scan_with(dead).is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn known_session_ends_when_process_dies() {
        let dir = temp_dir();
        place(&dir, "busy.json", "18504.json");
        let mut sd = SessionDir::new(dir.clone());
        sd.scan_with(alive_matching);
        let events = sd.scan_with(dead);
        assert_eq!(events, vec![SourceEvent::Ended { id: "11111111-2222-4333-8444-555555555555".into() }]);
        assert!(sd.scan_with(dead).is_empty(), "ended is reported once");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn known_session_ends_on_pid_reuse() {
        let dir = temp_dir();
        place(&dir, "busy.json", "18504.json");
        let mut sd = SessionDir::new(dir.clone());
        sd.scan_with(alive_matching);
        let events = sd.scan_with(|_| Proc::Alive { started: Some(BUSY_START + 3_600 * 10_000_000) });
        assert!(matches!(&events[..], [SourceEvent::Ended { .. }]));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn known_session_ends_when_file_removed() {
        let dir = temp_dir();
        place(&dir, "busy.json", "18504.json");
        place(&dir, "idle.json", "23116.json");
        let mut sd = SessionDir::new(dir.clone());
        assert_eq!(sd.scan_with(alive_unknown).len(), 2);
        std::fs::remove_file(dir.join("18504.json")).unwrap();
        let events = sd.scan_with(alive_unknown);
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], SourceEvent::Upsert { info, .. } if info.name == "sample-service-7"));
        assert_eq!(events[1], SourceEvent::Ended { id: "11111111-2222-4333-8444-555555555555".into() });
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn file_caught_mid_write_does_not_end_session() {
        let dir = temp_dir();
        place(&dir, "busy.json", "18504.json");
        let mut sd = SessionDir::new(dir.clone());
        sd.scan_with(alive_matching);
        std::fs::write(dir.join("18504.json"), b"{\"pid\":18504,\"sessi").unwrap();
        assert!(sd.scan_with(alive_matching).is_empty());
        place(&dir, "busy.json", "18504.json");
        assert_eq!(sd.scan_with(dead).len(), 1, "still known, so death is reported");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_directory_yields_nothing() {
        let mut sd = SessionDir::new(std::env::temp_dir().join("peepthis-test-does-not-exist"));
        assert!(sd.scan_with(alive_unknown).is_empty());
    }

    #[test]
    fn rescan_of_unchanged_dir_repeats_upserts_only() {
        let dir = temp_dir();
        place(&dir, "idle.json", "23116.json");
        let mut sd = SessionDir::new(dir.clone());
        let a = sd.scan_with(alive_unknown);
        let b = sd.scan_with(alive_unknown);
        assert_eq!(a, b);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
