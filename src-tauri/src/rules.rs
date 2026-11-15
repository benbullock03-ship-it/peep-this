//! Suggested auto-allow rules (spec §11.8): offered after three manual allows of one shape,
//! kept in `<dir>/rules.json`. The `destructive` tier is never offered and never matched.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::risk::{self, Tier};
use crate::trail;

pub const OFFER_AFTER: u32 = 3;
/// Stands for one or more relative file paths inside the project.
pub const FILES: &str = "<files>";
const MAX_COMMAND: usize = 300;
const VERSION: u32 = 1;
/// Anything that can chain, redirect, substitute or quote makes a command too clever to generalise.
const UNSAFE_CHARS: &[char] = &[';', '&', '|', '<', '>', '`', '$', '(', ')', '{', '}', '"', '\'', '*', '?', '%', '^', '!', '#', '@'];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub cwd: String,
    pub tool: String,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    #[serde(flatten)]
    pub shape: Shape,
    pub created: u64,
    #[serde(default)]
    pub uses: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    pub id: String,
    pub session_id: String,
    #[serde(flatten)]
    pub shape: Shape,
    pub tier: Tier,
    pub at: u64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub rules: Vec<Rule>,
    pub offers: Vec<Offer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Yes,
    NotNow,
    Never,
}

impl Answer {
    pub fn parse(s: &str) -> Option<Answer> {
        match s {
            "yes" => Some(Answer::Yes),
            "not-now" => Some(Answer::NotNow),
            "never" => Some(Answer::Never),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Book {
    version: u32,
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(default)]
    never: Vec<Shape>,
}

/// Lowercase, forward slashes, no quotes or trailing slash, Git Bash `/c/` as `c:/`.
fn norm_path(p: &str) -> String {
    let mut s = p.trim().trim_matches('"').replace('\\', "/").to_lowercase();
    while s.contains("//") {
        s = s.replace("//", "/");
    }
    let b = s.as_bytes();
    if b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b'/' {
        s = format!("{}:{}", b[1] as char, &s[2..]);
    }
    while s.len() > 1 && s.ends_with('/') && !s.ends_with(":/") {
        s.pop();
    }
    s
}

fn is_abs(p: &str) -> bool {
    p.starts_with('/') || p.as_bytes().get(1) == Some(&b':')
}

/// `ts` for `a.ts`; none for `Makefile`, `.env` or `1.2`.
fn ext_of(name: &str) -> Option<&str> {
    let (stem, ext) = name.rsplit_once('.')?;
    let ok = !stem.is_empty()
        && (1..=8).contains(&ext.len())
        && ext.chars().all(|c| c.is_ascii_alphanumeric())
        && ext.chars().any(|c| c.is_ascii_alphabetic());
    ok.then_some(ext)
}

fn in_project_file(word: &str) -> bool {
    let w = word.replace('\\', "/");
    if w.starts_with('-') || w.starts_with('~') || w.contains(':') || w.starts_with('/') || w.contains('=') {
        return false;
    }
    if w.split('/').any(|s| s == "..") {
        return false;
    }
    w.contains('/') || ext_of(&w).is_some()
}

/// Command and subcommand kept, every other word kept, trailing in-project file arguments folded into `<files>`.
pub fn command_pattern(cmd: &str) -> Option<String> {
    let cmd = cmd.trim();
    if cmd.is_empty() || cmd.len() > MAX_COMMAND || cmd.chars().any(|c| c.is_control() || UNSAFE_CHARS.contains(&c)) {
        return None;
    }
    let words: Vec<&str> = cmd.split_whitespace().collect();
    let mut keep = words.len();
    while keep > 2 && in_project_file(words[keep - 1]) {
        keep -= 1;
    }
    let mut out = words[..keep].join(" ");
    if keep < words.len() {
        out.push(' ');
        out.push_str(FILES);
    }
    Some(out)
}

/// `src/*.ts` for a file inside `cwd`; the exact path otherwise, or when the name has no extension.
pub fn path_pattern(path: &str, cwd: &str) -> Option<String> {
    let p = norm_path(path);
    if p.is_empty() {
        return None;
    }
    let rel = if is_abs(&p) { p.strip_prefix(cwd).and_then(|r| r.strip_prefix('/')) } else { Some(p.as_str()) };
    let Some(rel) = rel.filter(|r| !r.is_empty() && !r.split('/').any(|s| s == ".." || s == ".")) else {
        return Some(p.clone());
    };
    let (dir, name) = match rel.rsplit_once('/') {
        Some((d, n)) => (Some(d), n),
        None => (None, rel),
    };
    let Some(ext) = ext_of(name) else { return Some(rel.to_string()) };
    Some(match dir {
        Some(d) => format!("{d}/*.{ext}"),
        None => format!("*.{ext}"),
    })
}

/// The shape a rule is matched on. None when the request has nothing exact enough to generalise.
pub fn shape_of(tool: &str, input: &Value, cwd: &str) -> Option<Shape> {
    let cwd = norm_path(cwd);
    if cwd.is_empty() || tool.trim().is_empty() {
        return None;
    }
    let pattern = match tool {
        "Bash" | "PowerShell" => command_pattern(input.get("command")?.as_str()?)?,
        _ => match ["file_path", "notebook_path"].iter().find_map(|k| input.get(*k).and_then(Value::as_str)) {
            Some(p) => path_pattern(p, &cwd)?,
            None => {
                let t = trail::target_of(input);
                if t.is_empty() || t.chars().count() >= trail::TARGET_MAX_CHARS {
                    return None;
                }
                t
            }
        },
    };
    Some(Shape { cwd, tool: tool.to_string(), pattern })
}

/// A shape that may become a rule: never `destructive`.
fn offerable(tool: &str, input: &Value, cwd: &str) -> Option<(Shape, Tier)> {
    let tier = risk::classify(tool, input, cwd).tier;
    if tier == Tier::Destructive {
        return None;
    }
    Some((shape_of(tool, input, cwd)?, tier))
}

pub type OnChange = Box<dyn Fn(&View) + Send + Sync>;

#[derive(Default)]
struct Mem {
    counts: HashMap<Shape, u32>,
    offers: Vec<Offer>,
    seq: u64,
}

pub struct Rules {
    path: PathBuf,
    mem: Mutex<Mem>,
    on_change: OnceLock<OnChange>,
}

impl Rules {
    /// `base` is `%APPDATA%\PeepThis`.
    pub fn open(base: &Path) -> Arc<Rules> {
        Arc::new(Rules { path: base.join("rules.json"), mem: Mutex::new(Mem::default()), on_change: OnceLock::new() })
    }

    pub fn set_on_change(&self, f: OnChange) {
        let _ = self.on_change.set(f);
    }

    /// A missing file is an empty book; anything unreadable is an error, never an empty book.
    fn load(&self) -> Result<Book, String> {
        let text = match fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Book { version: VERSION, ..Book::default() }),
            Err(e) => return Err(format!("rules.json unreadable: {e}")),
        };
        let book: Book = serde_json::from_str(&text).map_err(|e| format!("rules.json is not valid: {e}"))?;
        if book.version != VERSION {
            return Err(format!("rules.json has unknown version {}", book.version));
        }
        Ok(book)
    }

    fn save(&self, book: &Book) -> Result<(), String> {
        let write = || -> std::io::Result<()> {
            if let Some(dir) = self.path.parent() {
                fs::create_dir_all(dir)?;
            }
            let tmp = self.path.with_extension("json.tmp");
            fs::write(&tmp, serde_json::to_vec_pretty(book)?)?;
            fs::rename(&tmp, &self.path)
        };
        write().map_err(|e| format!("rules.json not saved: {e}"))
    }

    fn view_of(&self, mem: &Mem) -> View {
        match self.load() {
            Ok(b) => View { rules: b.rules, offers: mem.offers.clone(), error: None },
            Err(e) => View { rules: Vec::new(), offers: Vec::new(), error: Some(e) },
        }
    }

    pub fn view(&self) -> View {
        let mem = self.mem.lock().unwrap();
        self.view_of(&mem)
    }

    fn changed(&self, view: &View) {
        if let Some(f) = self.on_change.get() {
            f(view);
        }
    }

    /// The id of the rule that allows this request. Re-classifies first; `destructive` and any file trouble mean none.
    pub fn auto_rule(&self, tool: &str, input: &Value, cwd: &str) -> Option<String> {
        let (shape, _) = offerable(tool, input, cwd)?;
        let _mem = self.mem.lock().unwrap();
        match self.load() {
            Ok(b) => b.rules.into_iter().find(|r| r.shape == shape).map(|r| r.id),
            Err(e) => {
                eprintln!("peepthis: no auto-allow: {e}");
                None
            }
        }
    }

    pub fn record_use(&self, id: &str, now: u64) {
        let view = {
            let mem = self.mem.lock().unwrap();
            let mut book = match self.load() {
                Ok(b) => b,
                Err(e) => return eprintln!("peepthis: rule use not counted: {e}"),
            };
            let Some(r) = book.rules.iter_mut().find(|r| r.id == id) else { return };
            r.uses += 1;
            r.last_used = Some(now);
            if let Err(e) = self.save(&book) {
                return eprintln!("peepthis: {e}");
            }
            self.view_of(&mem)
        };
        self.changed(&view);
    }

    /// A manual allow from the widget. True when this one raised an offer.
    pub fn note_allow(&self, session_id: &str, tool: &str, input: &Value, cwd: &str, now: u64) -> bool {
        let Some((shape, tier)) = offerable(tool, input, cwd) else { return false };
        let view = {
            let mut mem = self.mem.lock().unwrap();
            let book = match self.load() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("peepthis: no rule offer: {e}");
                    return false;
                }
            };
            if book.rules.iter().any(|r| r.shape == shape)
                || book.never.contains(&shape)
                || mem.offers.iter().any(|o| o.shape == shape)
            {
                return false;
            }
            let n = mem.counts.entry(shape.clone()).or_insert(0);
            *n += 1;
            if *n < OFFER_AFTER {
                return false;
            }
            mem.counts.remove(&shape);
            mem.seq += 1;
            let id = format!("of-{now}-{}", mem.seq);
            mem.offers.push(Offer { id, session_id: session_id.to_string(), shape, tier, at: now });
            self.view_of(&mem)
        };
        self.changed(&view);
        true
    }

    /// A manual deny starts the count for that shape again.
    pub fn note_deny(&self, tool: &str, input: &Value, cwd: &str) {
        if let Some(shape) = shape_of(tool, input, cwd) {
            self.mem.lock().unwrap().counts.remove(&shape);
        }
    }

    pub fn answer(&self, offer_id: &str, answer: Answer, now: u64) -> Result<View, String> {
        let view = {
            let mut mem = self.mem.lock().unwrap();
            let Some(i) = mem.offers.iter().position(|o| o.id == offer_id) else { return Ok(self.view_of(&mem)) };
            if answer != Answer::NotNow {
                let mut book = self.load()?;
                let shape = mem.offers[i].shape.clone();
                if answer == Answer::Yes && !book.rules.iter().any(|r| r.shape == shape) {
                    let mut id = format!("r{now:x}");
                    while book.rules.iter().any(|r| r.id == id) {
                        id.push('x');
                    }
                    book.rules.push(Rule { id, shape, created: now, uses: 0, last_used: None });
                } else if answer == Answer::Never && !book.never.contains(&shape) {
                    book.never.push(shape);
                }
                self.save(&book)?;
            }
            let offer = mem.offers.remove(i);
            mem.counts.remove(&offer.shape);
            self.view_of(&mem)
        };
        self.changed(&view);
        Ok(view)
    }

    pub fn revoke(&self, rule_id: &str) -> Result<View, String> {
        let view = {
            let mem = self.mem.lock().unwrap();
            let mut book = self.load()?;
            let before = book.rules.len();
            book.rules.retain(|r| r.id != rule_id);
            if book.rules.len() != before {
                self.save(&book)?;
            }
            self.view_of(&mem)
        };
        self.changed(&view);
        Ok(view)
    }
}

#[tauri::command]
pub fn rules_view(rules: tauri::State<'_, Arc<Rules>>) -> View {
    rules.view()
}

/// Rules are only ever created here, from an offer the backend raised after real allows.
#[tauri::command]
pub fn answer_rule_offer(offer_id: String, answer: String, rules: tauri::State<'_, Arc<Rules>>) -> Result<View, String> {
    let a = Answer::parse(&answer).ok_or_else(|| format!("unknown answer {answer:?}"))?;
    rules.answer(&offer_id, a, crate::store::now_ms())
}

#[tauri::command]
pub fn revoke_rule(rule_id: String, rules: tauri::State<'_, Arc<Rules>>) -> Result<View, String> {
    rules.revoke(&rule_id)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};

    pub const CWD: &str = "C:\\work\\demo";
    const T: u64 = 1_794_657_600_000;

    pub struct TempDir(pub PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    pub fn temp() -> TempDir {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "peepthis-rules-{}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst),
            crate::store::now_ms()
        ));
        fs::create_dir_all(&p).unwrap();
        TempDir(p)
    }

    fn bash(cmd: &str) -> Value {
        json!({ "command": cmd })
    }

    fn pat(tool: &str, input: Value) -> Option<String> {
        shape_of(tool, &input, CWD).map(|s| s.pattern)
    }

    fn allow_n(r: &Rules, n: usize, cmd: &str) -> Vec<bool> {
        (0..n).map(|i| r.note_allow("s1", "Bash", &bash(cmd), CWD, T + i as u64)).collect()
    }

    #[test]
    fn commands_keep_command_and_subcommand_and_fold_trailing_files() {
        assert_eq!(command_pattern("npm test").as_deref(), Some("npm test"));
        assert_eq!(command_pattern("  npm   test ").as_deref(), Some("npm test"));
        assert_eq!(command_pattern("npx vitest run src/a.test.ts").as_deref(), Some("npx vitest run <files>"));
        assert_eq!(command_pattern("npx vitest run src/a.test.ts src\\b.test.ts").as_deref(), Some("npx vitest run <files>"));
        assert_eq!(command_pattern("git add src/a.ts").as_deref(), Some("git add <files>"));
        assert_eq!(command_pattern("cargo test parse").as_deref(), Some("cargo test parse"));
        assert_eq!(command_pattern("git diff --stat").as_deref(), Some("git diff --stat"));
        // Command and subcommand are never folded, even when they look like files.
        assert_eq!(command_pattern("node script.js").as_deref(), Some("node script.js"));
        assert_eq!(command_pattern("cat a.ts").as_deref(), Some("cat a.ts"));
        // Files outside the project stay literal, so they never match a `<files>` rule.
        assert_eq!(command_pattern("git add ../other/a.ts").as_deref(), Some("git add ../other/a.ts"));
        assert_eq!(command_pattern("git add C:/x/a.ts").as_deref(), Some("git add C:/x/a.ts"));
        assert_eq!(command_pattern("git add /etc/a.conf").as_deref(), Some("git add /etc/a.conf"));
        assert_eq!(command_pattern("npm version 1.2").as_deref(), Some("npm version 1.2"));
        for clever in ["npm test && rm x", "echo $HOME", "ls | sh", "a; b", "git commit -m \"x\"", "rm *.ts", "a\nb", "cat > a.ts", ""] {
            assert_eq!(command_pattern(clever), None, "{clever:?}");
        }
    }

    #[test]
    fn paths_become_a_folder_glob_inside_the_project() {
        assert_eq!(pat("Edit", json!({ "file_path": "C:\\work\\demo\\src\\a.ts" })).as_deref(), Some("src/*.ts"));
        assert_eq!(pat("Write", json!({ "file_path": "c:/WORK/demo/README.md" })).as_deref(), Some("*.md"));
        assert_eq!(pat("Edit", json!({ "file_path": "/c/work/demo/src/b.ts" })).as_deref(), Some("src/*.ts"));
        assert_eq!(pat("Edit", json!({ "file_path": "C:\\work\\demo\\Makefile" })).as_deref(), Some("makefile"));
        assert_eq!(pat("Edit", json!({ "file_path": "C:\\work\\demo2\\a.ts" })).as_deref(), Some("c:/work/demo2/a.ts"));
        assert_eq!(pat("Edit", json!({ "file_path": "C:\\work\\demo\\..\\x\\a.ts" })).as_deref(), Some("c:/work/demo/../x/a.ts"));
        assert_eq!(pat("WebFetch", json!({ "url": "https://example.com" })).as_deref(), Some("https://example.com"));
        assert_eq!(pat("mcp__x__y", json!({ "server": 1 })), None);
        let s = shape_of("Bash", &bash("npm test"), "C:\\Work\\Demo\\").unwrap();
        assert_eq!(s, Shape { cwd: "c:/work/demo".into(), tool: "Bash".into(), pattern: "npm test".into() });
    }

    #[test]
    fn the_third_allow_offers_not_the_second() {
        let d = temp();
        let r = Rules::open(&d.0);
        assert_eq!(allow_n(&r, 2, "npm test"), vec![false, false]);
        assert!(r.view().offers.is_empty());
        assert!(r.note_allow("s9", "Bash", &bash("npm  test"), CWD, T + 5));
        let v = r.view();
        assert_eq!(v.offers.len(), 1);
        assert_eq!((v.offers[0].session_id.as_str(), v.offers[0].shape.pattern.as_str()), ("s9", "npm test"));
        assert!(!r.note_allow("s1", "Bash", &bash("npm test"), CWD, T + 6), "one open offer per shape");
    }

    #[test]
    fn other_projects_tools_and_targets_count_apart_and_a_deny_restarts() {
        let d = temp();
        let r = Rules::open(&d.0);
        r.note_allow("s1", "Bash", &bash("npm test"), CWD, T);
        r.note_allow("s1", "Bash", &bash("npm test"), "C:\\work\\other", T);
        r.note_allow("s1", "PowerShell", &bash("npm test"), CWD, T);
        r.note_allow("s1", "Bash", &bash("npm run build"), CWD, T);
        r.note_allow("s1", "Bash", &bash("npm test"), CWD, T);
        r.note_deny("Bash", &bash("npm test"), CWD);
        assert!(!r.note_allow("s1", "Bash", &bash("npm test"), CWD, T));
        assert!(r.view().offers.is_empty());
    }

    #[test]
    fn destructive_is_never_offered_and_never_matched() {
        let d = temp();
        let r = Rules::open(&d.0);
        let push = bash("git push --force");
        assert_eq!(risk::classify("Bash", &push, CWD).tier, Tier::Destructive);
        for _ in 0..5 {
            assert!(!r.note_allow("s1", "Bash", &push, CWD, T));
        }
        assert!(r.view().offers.is_empty());
        // A hand-edited rule for that exact shape still does nothing.
        let shape = shape_of("Bash", &push, CWD).unwrap();
        let book = json!({ "version": 1, "rules": [{ "id": "r1", "cwd": shape.cwd, "tool": "Bash", "pattern": shape.pattern, "created": T }] });
        fs::write(d.0.join("rules.json"), book.to_string()).unwrap();
        assert_eq!(r.view().rules.len(), 1);
        assert_eq!(r.auto_rule("Bash", &push, CWD), None);
    }

    #[test]
    fn yes_makes_a_rule_that_matches_exactly_and_counts_uses() {
        let d = temp();
        let r = Rules::open(&d.0);
        allow_n(&r, 3, "npx vitest run src/a.test.ts");
        let offer = r.view().offers[0].clone();
        assert_eq!(offer.shape.pattern, "npx vitest run <files>");
        let v = r.answer(&offer.id, Answer::Yes, T + 10).unwrap();
        assert!(v.offers.is_empty());
        let id = v.rules[0].id.clone();
        assert_eq!(r.auto_rule("Bash", &bash("npx vitest run src/other.test.ts lib/x.ts"), CWD), Some(id.clone()));
        assert_eq!(r.auto_rule("Bash", &bash("npx vitest run"), CWD), None);
        assert_eq!(r.auto_rule("Bash", &bash("npx vitest run ../x.test.ts"), CWD), None);
        assert_eq!(r.auto_rule("Bash", &bash("npx vitest run src/a.test.ts"), "C:\\work\\other"), None);
        assert_eq!(r.auto_rule("PowerShell", &bash("npx vitest run src/a.test.ts"), CWD), None);
        r.record_use(&id, T + 20);
        let rule = &r.view().rules[0];
        assert_eq!((rule.uses, rule.last_used), (1, Some(T + 20)));
        let on_disk: Value = serde_json::from_str(&fs::read_to_string(d.0.join("rules.json")).unwrap()).unwrap();
        assert_eq!(on_disk["rules"][0]["pattern"], "npx vitest run <files>");
        assert_eq!(on_disk["rules"][0]["cwd"], "c:/work/demo");
        assert!(!r.note_allow("s1", "Bash", &bash("npx vitest run src/b.test.ts"), CWD, T), "a ruled shape is not offered again");
    }

    #[test]
    fn revoke_removes_the_rule_and_it_stops_matching() {
        let d = temp();
        let r = Rules::open(&d.0);
        allow_n(&r, 3, "npm test");
        let offer = r.view().offers[0].id.clone();
        let id = r.answer(&offer, Answer::Yes, T).unwrap().rules[0].id.clone();
        assert!(r.auto_rule("Bash", &bash("npm test"), CWD).is_some());
        assert!(r.revoke(&id).unwrap().rules.is_empty());
        assert_eq!(r.auto_rule("Bash", &bash("npm test"), CWD), None);
        assert!(r.revoke(&id).is_ok(), "revoking twice is harmless");
        // Revoked shapes can be offered again after three more allows.
        assert_eq!(allow_n(&r, 3, "npm test"), vec![false, false, true]);
    }

    #[test]
    fn never_ask_suppresses_the_offer_for_good_and_not_now_restarts_the_count() {
        let d = temp();
        let r = Rules::open(&d.0);
        allow_n(&r, 3, "npm test");
        let offer = r.view().offers[0].id.clone();
        r.answer(&offer, Answer::NotNow, T).unwrap();
        assert!(r.view().offers.is_empty());
        assert_eq!(allow_n(&r, 3, "npm test"), vec![false, false, true]);
        let offer = r.view().offers[0].id.clone();
        r.answer(&offer, Answer::Never, T).unwrap();
        assert_eq!(allow_n(&r, 6, "npm test"), vec![false; 6]);
        assert!(r.view().rules.is_empty());
        assert_eq!(r.auto_rule("Bash", &bash("npm test"), CWD), None);
        let reopened = Rules::open(&d.0);
        assert_eq!(allow_n(&reopened, 3, "npm test"), vec![false; 3], "never-ask survives a restart");
    }

    #[test]
    fn a_corrupt_file_means_no_auto_allow_no_offer_and_no_overwrite() {
        let d = temp();
        let r = Rules::open(&d.0);
        allow_n(&r, 3, "npm test");
        let offer = r.view().offers[0].id.clone();
        r.answer(&offer, Answer::Yes, T).unwrap();
        assert!(r.auto_rule("Bash", &bash("npm test"), CWD).is_some());
        let path = d.0.join("rules.json");
        let good = fs::read_to_string(&path).unwrap();
        for bad in [&good[..good.len() / 2], "", "{\"version\":2,\"rules\":[]}", "{\"version\":1,\"rules\":[{\"id\":1}]}"] {
            fs::write(&path, bad).unwrap();
            assert_eq!(r.auto_rule("Bash", &bash("npm test"), CWD), None, "{bad:?}");
            assert!(r.view().error.is_some());
            assert_eq!(allow_n(&r, 3, "npm run build"), vec![false; 3]);
            assert!(r.revoke("anything").is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), bad, "the user's file is left alone");
        }
    }

    #[test]
    fn an_unreadable_path_fails_closed() {
        let d = temp();
        fs::create_dir_all(d.0.join("rules.json")).unwrap();
        let r = Rules::open(&d.0);
        assert_eq!(r.auto_rule("Bash", &bash("npm test"), CWD), None);
        assert!(r.view().error.is_some());
    }

    #[test]
    fn a_missing_file_is_an_empty_book() {
        let d = temp();
        let r = Rules::open(&d.0);
        assert_eq!(r.view(), View::default());
        assert!(!d.0.join("rules.json").exists(), "reading never creates the file");
    }

    #[test]
    fn answers_parse() {
        assert_eq!(Answer::parse("yes"), Some(Answer::Yes));
        assert_eq!(Answer::parse("not-now"), Some(Answer::NotNow));
        assert_eq!(Answer::parse("never"), Some(Answer::Never));
        assert_eq!(Answer::parse("always"), None);
    }
}
