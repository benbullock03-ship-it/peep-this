//! Decision trail (spec §11.5): one JSONL line per approval outcome in `<dir>/trail/<yyyy-mm>.jsonl`.
//! Commands and paths only; tool content (diffs, file text) is never written.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::approve::{Decision, Release};
use crate::risk::Tier;

pub const RETENTION_DAYS: u64 = 90;
const DAY_MS: u64 = 86_400_000;
/// Long heredoc commands would otherwise put file text on disk.
pub const TARGET_MAX_CHARS: usize = 500;
const REASON_MAX_CHARS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Allowed,
    Denied,
    Released,
    AutoAllowed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub t: u64,
    pub session_id: String,
    pub session: String,
    pub cwd: String,
    pub tool: String,
    #[serde(default)]
    pub target: String,
    pub tier: Tier,
    pub outcome: Outcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// Why a request was released, e.g. `timed-out`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

/// The command or path a request acts on; empty for tools with neither.
pub fn target_of(input: &Value) -> String {
    ["command", "file_path", "notebook_path", "path", "url", "pattern", "query"]
        .iter()
        .find_map(|k| input.get(*k).and_then(Value::as_str).filter(|s| !s.trim().is_empty()))
        .map(|s| clip(s, TARGET_MAX_CHARS))
        .unwrap_or_default()
}

pub fn via(r: Release) -> &'static str {
    match r {
        Release::Window => "timed-out",
        Release::Foreground => "terminal-focused",
        Release::LeftWaiting => "moved-on",
        Release::SessionGone => "session-ended",
        Release::Disconnected => "answered-in-terminal",
        Release::Hidden => "widget-hidden",
        Release::Quit => "widget-quit",
    }
}

impl Entry {
    pub fn from_decision(d: &Decision<'_>, session_name: &str) -> Entry {
        Entry {
            t: d.at,
            session_id: d.session_id.to_string(),
            session: session_name.to_string(),
            cwd: d.request.cwd.clone(),
            tool: d.request.tool.clone(),
            target: target_of(&d.request.input),
            tier: d.request.risk.tier,
            outcome: d.outcome,
            reason: d.reason.map(|r| clip(r, REASON_MAX_CHARS)),
            rule: d.rule.map(str::to_owned),
            via: d.via.map(|r| via(r).to_string()),
            agent_id: d.request.agent_id.clone(),
        }
    }
}

/// Civil date (UTC) for days since 1970-01-01 (Hinnant's algorithm).
fn civil(days: i64) -> (i64, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m)
}

fn days_from_civil(y: i64, m: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
    let doy = (153 * mp + 2) / 5;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `2026-11` for a UTC epoch-ms time.
pub fn month_of(ms: u64) -> String {
    let (y, m) = civil((ms / DAY_MS) as i64);
    format!("{y:04}-{m:02}")
}

/// Epoch ms of the first instant after the month named by a `yyyy-mm` file stem.
fn month_end_ms(stem: &str) -> Option<u64> {
    let (y, m) = stem.split_once('-')?;
    let (y, m): (i64, u32) = (y.parse().ok()?, m.parse().ok()?);
    if !(1..=12).contains(&m) || stem.len() != 7 {
        return None;
    }
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    u64::try_from(days_from_civil(ny, nm)).ok().map(|d| d * DAY_MS)
}

fn parse_lines(text: &str) -> impl Iterator<Item = Entry> + '_ {
    text.lines().filter(|l| !l.trim().is_empty()).filter_map(|l| match serde_json::from_str::<Entry>(l) {
        Ok(e) => Some(e),
        Err(e) => {
            eprintln!("peepthis: trail line skipped: {e}");
            None
        }
    })
}

pub struct Trail {
    dir: PathBuf,
    settings: PathBuf,
    enabled: AtomicBool,
    io: Mutex<()>,
}

impl Trail {
    /// `base` is `%APPDATA%\PeepThis`; the on/off switch lives in its `settings.json` under `trail`.
    pub fn open(base: &Path) -> Arc<Trail> {
        let settings = base.join("settings.json");
        let enabled = read_settings(&settings).get("trail").and_then(Value::as_bool).unwrap_or(true);
        Arc::new(Trail { dir: base.join("trail"), settings, enabled: AtomicBool::new(enabled), io: Mutex::new(()) })
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn set_enabled(&self, on: bool) -> io::Result<()> {
        let mut v = read_settings(&self.settings);
        if !v.is_object() {
            v = Value::Object(Default::default());
        }
        v["trail"] = Value::Bool(on);
        if let Some(dir) = self.settings.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = self.settings.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(&v)?)?;
        fs::rename(&tmp, &self.settings)?;
        self.enabled.store(on, Ordering::SeqCst);
        Ok(())
    }

    /// `Ok(false)` when the trail is switched off and nothing was written.
    pub fn record(&self, e: &Entry) -> io::Result<bool> {
        if !self.enabled() {
            return Ok(false);
        }
        let mut line = serde_json::to_string(e)?;
        line.push('\n');
        let _g = self.io.lock().unwrap();
        fs::create_dir_all(&self.dir)?;
        let mut f = OpenOptions::new().create(true).append(true).open(self.dir.join(format!("{}.jsonl", month_of(e.t))))?;
        f.write_all(line.as_bytes())?;
        Ok(true)
    }

    fn files(&self) -> Vec<(String, PathBuf)> {
        let Ok(rd) = fs::read_dir(&self.dir) else { return Vec::new() };
        let mut out: Vec<(String, PathBuf)> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
            .filter_map(|p| {
                let stem = p.file_stem()?.to_str()?.to_string();
                month_end_ms(&stem).map(|_| (stem, p))
            })
            .collect();
        out.sort();
        out
    }

    /// Every readable entry, newest first. Malformed lines are skipped.
    pub fn read_all(&self) -> Vec<Entry> {
        let _g = self.io.lock().unwrap();
        let mut out: Vec<Entry> = Vec::new();
        for (_, p) in self.files() {
            match fs::read_to_string(&p) {
                Ok(text) => out.extend(parse_lines(&text)),
                Err(e) => eprintln!("peepthis: trail file {} unreadable: {e}", p.display()),
            }
        }
        out.sort_by(|a, b| b.t.cmp(&a.t));
        out
    }

    /// The last `n` entries of each session, newest first.
    pub fn recent(&self, n: usize) -> HashMap<String, Vec<Entry>> {
        let mut out: HashMap<String, Vec<Entry>> = HashMap::new();
        for e in self.read_all() {
            let list = out.entry(e.session_id.clone()).or_default();
            if list.len() < n {
                list.push(e);
            }
        }
        out
    }

    /// Drops entries older than `RETENTION_DAYS`: whole months by deleting the file, the boundary month by rewriting it.
    pub fn prune(&self, now: u64) -> io::Result<usize> {
        let cutoff = now.saturating_sub(RETENTION_DAYS * DAY_MS);
        let _g = self.io.lock().unwrap();
        let mut dropped = 0;
        for (stem, p) in self.files() {
            let Some(end) = month_end_ms(&stem) else { continue };
            if end <= cutoff {
                let text = fs::read_to_string(&p).unwrap_or_default();
                dropped += text.lines().filter(|l| !l.trim().is_empty()).count();
                fs::remove_file(&p)?;
                continue;
            }
            let text = fs::read_to_string(&p)?;
            let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
            let keep: Vec<&str> = lines
                .iter()
                .copied()
                .filter(|l| serde_json::from_str::<Entry>(l).is_ok_and(|e| e.t >= cutoff))
                .collect();
            if keep.len() == lines.len() {
                continue;
            }
            dropped += lines.len() - keep.len();
            let tmp = p.with_extension("jsonl.tmp");
            let mut body = keep.join("\n");
            if !body.is_empty() {
                body.push('\n');
            }
            fs::write(&tmp, body)?;
            fs::rename(&tmp, &p)?;
        }
        Ok(dropped)
    }
}

fn read_settings(path: &Path) -> Value {
    fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null)
}

#[tauri::command]
pub fn trail_log(trail: tauri::State<'_, Arc<Trail>>) -> Vec<Entry> {
    trail.read_all()
}

#[tauri::command]
pub fn trail_recent(per_session: usize, trail: tauri::State<'_, Arc<Trail>>) -> HashMap<String, Vec<Entry>> {
    trail.recent(per_session)
}

#[tauri::command]
pub fn trail_enabled(trail: tauri::State<'_, Arc<Trail>>) -> bool {
    trail.enabled()
}

#[tauri::command]
pub fn set_trail_enabled(on: bool, trail: tauri::State<'_, Arc<Trail>>) -> Result<bool, String> {
    trail.set_enabled(on).map_err(|e| e.to_string())?;
    Ok(trail.enabled())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approve::PendingApproval;
    use crate::risk;
    use serde_json::json;
    use std::sync::atomic::AtomicU32;

    /// 2026-11-14T12:00:00Z
    const NOW: u64 = 1_794_657_600_000;

    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn temp() -> TempDir {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "peepthis-trail-{}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst),
            crate::store::now_ms()
        ));
        fs::create_dir_all(&p).unwrap();
        TempDir(p)
    }

    fn entry(t: u64, sid: &str, outcome: Outcome) -> Entry {
        Entry {
            t,
            session_id: sid.into(),
            session: format!("name-{sid}"),
            cwd: "C:\\work\\demo".into(),
            tool: "Bash".into(),
            target: "npm test".into(),
            tier: Tier::Shell,
            outcome,
            reason: (outcome == Outcome::Denied).then(|| "not now".into()),
            rule: None,
            via: (outcome == Outcome::Released).then(|| "timed-out".into()),
            agent_id: None,
        }
    }

    #[test]
    fn months_are_utc_calendar_months() {
        assert_eq!(month_of(NOW), "2026-11");
        assert_eq!(month_of(0), "1970-01");
        assert_eq!(month_end_ms("2026-11"), Some(1_796_083_200_000)); // 2026-12-01
        assert_eq!(month_of(1_796_083_200_000 - 1), "2026-11");
        assert_eq!(month_of(1_796_083_200_000), "2026-12");
        assert_eq!(month_end_ms("2026-12").map(month_of), Some("2027-01".into()));
        assert_eq!(month_end_ms("2026-13"), None);
        assert_eq!(month_end_ms("notes"), None);
    }

    #[test]
    fn write_then_read_back_newest_first() {
        let d = temp();
        let trail = Trail::open(&d.0);
        let a = entry(NOW - 2_000, "s1", Outcome::Allowed);
        let b = entry(NOW - 1_000, "s1", Outcome::Denied);
        let c = entry(NOW, "s2", Outcome::Released);
        for e in [&a, &b, &c] {
            assert!(trail.record(e).unwrap());
        }
        assert!(d.0.join("trail").join("2026-11.jsonl").is_file());
        assert_eq!(trail.read_all(), vec![c.clone(), b.clone(), a.clone()]);
        let line = fs::read_to_string(d.0.join("trail/2026-11.jsonl")).unwrap();
        let first: Value = serde_json::from_str(line.lines().nth(1).unwrap()).unwrap();
        assert_eq!(first["outcome"], "denied");
        assert_eq!(first["reason"], "not now");
        assert_eq!(first["tier"], "shell");
        assert_eq!(first["sessionId"], "s1");
    }

    #[test]
    fn entries_land_in_their_own_month_and_read_across_files() {
        let d = temp();
        let trail = Trail::open(&d.0);
        let oct = entry(NOW - 40 * DAY_MS, "s1", Outcome::Allowed);
        let nov = entry(NOW, "s1", Outcome::Allowed);
        trail.record(&oct).unwrap();
        trail.record(&nov).unwrap();
        assert!(d.0.join("trail/2026-10.jsonl").is_file());
        assert_eq!(trail.read_all(), vec![nov, oct]);
    }

    #[test]
    fn recent_caps_per_session() {
        let d = temp();
        let trail = Trail::open(&d.0);
        for i in 0..7 {
            trail.record(&entry(NOW + i, "s1", Outcome::Allowed)).unwrap();
        }
        trail.record(&entry(NOW, "s2", Outcome::Denied)).unwrap();
        let r = trail.recent(5);
        assert_eq!(r["s1"].len(), 5);
        assert_eq!(r["s1"][0].t, NOW + 6);
        assert_eq!(r["s2"].len(), 1);
    }

    #[test]
    fn malformed_lines_are_skipped() {
        let d = temp();
        let trail = Trail::open(&d.0);
        let good = entry(NOW, "s1", Outcome::Allowed);
        trail.record(&good).unwrap();
        let path = d.0.join("trail/2026-11.jsonl");
        let mut f = OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(b"{not json\n\n{\"t\":5}\n").unwrap();
        trail.record(&entry(NOW + 1, "s1", Outcome::Denied)).unwrap();
        fs::write(d.0.join("trail/readme.jsonl"), "{}").unwrap();
        let all = trail.read_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[1], good);
    }

    #[test]
    fn prune_drops_old_months_and_old_lines_of_the_boundary_month() {
        let d = temp();
        let trail = Trail::open(&d.0);
        let ancient = entry(NOW - 200 * DAY_MS, "s1", Outcome::Allowed);
        let stale = entry(NOW - 91 * DAY_MS, "s1", Outcome::Denied); // 2026-08-15
        let kept_aug = entry(NOW - 89 * DAY_MS, "s1", Outcome::Allowed); // 2026-08-17
        let fresh = entry(NOW, "s1", Outcome::Released);
        for e in [&ancient, &stale, &kept_aug, &fresh] {
            trail.record(e).unwrap();
        }
        let aug = d.0.join("trail/2026-08.jsonl");
        let mut f = OpenOptions::new().append(true).open(&aug).unwrap();
        f.write_all(b"garbage\n").unwrap();
        drop(f);
        assert!(d.0.join(format!("trail/{}.jsonl", month_of(ancient.t))).is_file());

        assert_eq!(trail.prune(NOW).unwrap(), 3);
        assert!(!d.0.join(format!("trail/{}.jsonl", month_of(ancient.t))).exists());
        assert_eq!(trail.read_all(), vec![fresh, kept_aug]);
        assert_eq!(trail.prune(NOW).unwrap(), 0, "a second prune finds nothing");
    }

    #[test]
    fn switched_off_writes_nothing_and_the_setting_persists() {
        let d = temp();
        fs::write(d.0.join("settings.json"), r#"{"other":1}"#).unwrap();
        let trail = Trail::open(&d.0);
        assert!(trail.enabled());
        trail.set_enabled(false).unwrap();
        assert!(!trail.record(&entry(NOW, "s1", Outcome::Allowed)).unwrap());
        assert!(trail.read_all().is_empty());
        let v: Value = serde_json::from_str(&fs::read_to_string(d.0.join("settings.json")).unwrap()).unwrap();
        assert_eq!(v, json!({ "other": 1, "trail": false }));
        assert!(!Trail::open(&d.0).enabled());
        trail.set_enabled(true).unwrap();
        assert!(trail.record(&entry(NOW, "s1", Outcome::Allowed)).unwrap());
    }

    #[test]
    fn from_decision_keeps_the_target_never_the_content() {
        let input = json!({
            "file_path": "C:\\work\\demo\\src\\a.ts",
            "old_string": "secret old text",
            "new_string": "secret new text",
            "content": "whole file body"
        });
        let p = PendingApproval {
            id: "ap-1".into(),
            tool: "Edit".into(),
            input: input.clone(),
            cwd: "C:\\work\\demo".into(),
            agent_id: Some("a1".into()),
            received_at: NOW,
            release_at: NOW + 60_000,
            risk: risk::classify("Edit", &input, "C:\\work\\demo"),
        };
        let d = Decision { session_id: "s1", request: &p, outcome: Outcome::Released, reason: None, via: Some(Release::Window), rule: None, at: NOW };
        let e = Entry::from_decision(&d, "demo");
        assert_eq!(e.target, "C:\\work\\demo\\src\\a.ts");
        assert_eq!(e.via.as_deref(), Some("timed-out"));
        assert_eq!(e.tier, Tier::Safe);
        let line = serde_json::to_string(&e).unwrap();
        assert!(!line.contains("secret") && !line.contains("whole file"), "{line}");

        let long = "x".repeat(TARGET_MAX_CHARS + 50);
        assert_eq!(target_of(&json!({ "command": long })).chars().count(), TARGET_MAX_CHARS);
        assert_eq!(target_of(&json!({ "server": "mcp" })), "");
    }
}
