//! While-you-were-away digest (spec §11.9): presence from lock and idle, a capped in-memory activity log
//! per session, and the card built from the two. Events only; no model call.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tauri::{Emitter, Manager};

use crate::state::{Session, ShellState, State};
use crate::store::{self, Store};

pub const AWAY_AFTER_MIN_DEFAULT: u64 = 5;
pub const AWAY_AFTER_CHOICES: [u64; 6] = [1, 2, 5, 10, 15, 30];
/// Notes kept per session; older ones are dropped and the digest says its counts are partial.
pub const NOTES_KEPT: usize = 256;
pub const LINE_MAX_CHARS: usize = 160;
pub const PATH_MAX_CHARS: usize = 200;
/// File names listed per entry; the count covers the rest.
pub const FILES_LISTED: usize = 8;
/// Logs of sessions no longer in the store are kept this long after their last note.
pub const GONE_KEEP_MS: u64 = 24 * 60 * 60 * 1000;
const POLL: Duration = Duration::from_secs(1);
const STARTUP_GRACE_MS: u64 = 10_000;
const SETTINGS_KEY: &str = "awayAfterMin";
const SUMMARY_KEY: &str = "digestSummary";
/// Dev and live checks: overrides the setting with a threshold in seconds.
const ENV_SECS: &str = "PEEPTHIS_AWAY_SECS";

/// One thing a session did, as kept in the log: paths and counts, plus the last assistant line.
#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    Prompt,
    /// A file a tool changed (`Edit`, `Write`, `MultiEdit`, `NotebookEdit`), relative to the session cwd when inside it.
    File(String),
    Command,
    /// A held approval went to the terminal prompt.
    Released,
    Stop { line: Option<String> },
}

#[derive(Debug, Clone, Default)]
struct Log {
    name: String,
    notes: VecDeque<(u64, Note)>,
    /// Time of the newest note dropped for the cap.
    dropped_at: Option<u64>,
}

#[derive(Debug, Default)]
pub struct Activity {
    logs: HashMap<String, Log>,
}

impl Activity {
    pub fn note(&mut self, id: &str, name: &str, note: Note, now: u64) {
        let log = self.logs.entry(id.to_string()).or_default();
        if !name.is_empty() {
            log.name = name.to_string();
        }
        let note = match note {
            Note::File(p) => Note::File(clip(&p, PATH_MAX_CHARS)),
            Note::Stop { line } => Note::Stop { line: line.map(|l| clip(&l, LINE_MAX_CHARS)).filter(|l| !l.is_empty()) },
            n => n,
        };
        log.notes.push_back((now, note));
        while log.notes.len() > NOTES_KEPT {
            if let Some((t, _)) = log.notes.pop_front() {
                log.dropped_at = Some(t);
            }
        }
    }

    /// Drops logs of sessions the store no longer has once they are a day quiet.
    pub fn prune(&mut self, known: impl Fn(&str) -> bool, now: u64) {
        self.logs.retain(|id, log| {
            known(id) || log.notes.back().is_some_and(|(t, _)| now.saturating_sub(*t) < GONE_KEEP_MS)
        });
    }

    #[cfg(test)]
    fn len(&self, id: &str) -> usize {
        self.logs.get(id).map_or(0, |l| l.notes.len())
    }
}

/// First non-blank line, trimmed, at most `max` characters with an ellipsis.
pub(crate) fn clip(s: &str, max: usize) -> String {
    let line = s.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let mut out: String = line.chars().take(max).collect();
    if line.chars().count() > max {
        out.pop();
        out.push('…');
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    Blocked,
    Failed,
    Finished,
    Running,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub session_id: String,
    pub name: String,
    pub group: Group,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    pub alive: bool,
    /// Blocked on an approval the widget holds (the inbox can take it), not a question.
    pub pending: bool,
    pub took_ms: u64,
    pub files: Vec<String>,
    pub file_count: usize,
    pub commands: u32,
    pub released: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_line: Option<String>,
    /// The log's cap dropped notes inside the window: counts are a floor.
    pub partial: bool,
    /// One model-written sentence (AV-33), filled in after the card is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip)]
    last_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Digest {
    pub since: u64,
    pub until: u64,
    /// Blocked, failed, finished, running; inside a group the longest-waiting blocked entry first, else the latest activity first.
    pub entries: Vec<Entry>,
}

fn group_of(s: Option<&Session>, since: u64) -> Group {
    let Some(s) = s else { return Group::Finished };
    if !s.alive {
        return Group::Finished;
    }
    if s.state == State::Waiting || !s.pending.is_empty() {
        return Group::Blocked;
    }
    let shell_failed = s.shells.iter().any(|sh| sh.state == ShellState::Failed && sh.ended_at.is_some_and(|t| t >= since));
    match s.state {
        State::Error | State::Stuck => Group::Failed,
        _ if shell_failed => Group::Failed,
        State::Thinking | State::Working => Group::Running,
        _ => Group::Finished,
    }
}

/// The card for the window `since..until`, or `None` when nothing changed in it.
pub fn digest(activity: &Activity, sessions: &[Session], since: u64, until: u64) -> Option<Digest> {
    let by_id: HashMap<&str, &Session> = sessions.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut ids: Vec<&str> = activity
        .logs
        .iter()
        .filter(|(_, l)| l.notes.iter().any(|(t, _)| *t >= since))
        .map(|(id, _)| id.as_str())
        .collect();
    let seen: HashSet<&str> = ids.iter().copied().collect();
    ids.extend(sessions.iter().filter(|s| s.state_since >= since && !seen.contains(s.id.as_str())).map(|s| s.id.as_str()));

    let mut entries: Vec<Entry> = ids
        .into_iter()
        .map(|id| entry(id, activity.logs.get(id), by_id.get(id).copied(), since, until))
        .collect();
    if entries.is_empty() {
        return None;
    }
    entries.sort_by(|a, b| {
        a.group.cmp(&b.group).then_with(|| match a.group {
            Group::Blocked => a.last_at.cmp(&b.last_at),
            _ => b.last_at.cmp(&a.last_at),
        })
        .then_with(|| a.name.cmp(&b.name))
    });
    Some(Digest { since, until, entries })
}

fn entry(id: &str, log: Option<&Log>, s: Option<&Session>, since: u64, until: u64) -> Entry {
    let group = group_of(s, since);
    let empty = VecDeque::new();
    let notes = log.map_or(&empty, |l| &l.notes);
    // The turn that was running when the user left counts from its prompt.
    let mut open_turn = None;
    for (t, n) in notes.iter().take_while(|(t, _)| *t < since) {
        match n {
            Note::Prompt => open_turn = open_turn.or(Some(*t)),
            Note::Stop { .. } => open_turn = None,
            _ => {}
        }
    }
    let window: Vec<&(u64, Note)> = notes.iter().filter(|(t, _)| *t >= since).collect();
    let mut files: Vec<String> = Vec::new();
    let (mut commands, mut released, mut last_line) = (0, 0, None);
    for (_, n) in &window {
        match n {
            Note::File(p) => {
                files.retain(|f| f != p);
                files.push(p.clone());
            }
            Note::Command => commands += 1,
            Note::Released => released += 1,
            Note::Stop { line: Some(l) } => last_line = Some(l.clone()),
            _ => {}
        }
    }
    let file_count = files.len();
    files.truncate(FILES_LISTED);
    let state_at = s.map(|s| s.state_since).filter(|t| *t >= since);
    let last_note = window.last().map(|(t, _)| *t);
    let last_at = match group {
        Group::Blocked => s.map_or(since, |s| s.state_since),
        _ => last_note.into_iter().chain(state_at).max().unwrap_or(since),
    };
    let start = open_turn.or(window.first().map(|(t, _)| *t)).or(state_at).unwrap_or(since);
    let end = match group {
        Group::Finished => s.and_then(|s| s.ended_at).into_iter().chain(last_note).chain(state_at).max().unwrap_or(until),
        _ => until,
    };
    let name = s.map(|s| s.name.clone()).filter(|n| !n.is_empty()).or_else(|| log.map(|l| l.name.clone())).unwrap_or_default();
    Entry {
        session_id: id.to_string(),
        name: if name.is_empty() { id.chars().take(8).collect() } else { name },
        group,
        state: s.map(|s| s.state),
        alive: s.is_some_and(|s| s.alive),
        pending: s.is_some_and(|s| s.alive && !s.pending.is_empty()),
        took_ms: end.saturating_sub(start),
        files,
        file_count,
        commands,
        released,
        last_line,
        partial: log.and_then(|l| l.dropped_at).is_some_and(|t| t >= since),
        summary: None,
        last_at,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Here,
    /// `since` is the last input before leaving; `locked` once the screen locked during the absence.
    Away { since: u64, locked: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Probe {
    pub locked: bool,
    pub idle_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Left { since: u64 },
    Back { since: u64 },
}

/// Away: screen locked, or no input for `after_ms`. Back: unlock, or the first input.
pub fn step(p: Presence, probe: Probe, now: u64, after_ms: u64) -> (Presence, Option<Turn>) {
    let last_input = now.saturating_sub(probe.idle_ms);
    match p {
        Presence::Here if probe.locked || probe.idle_ms >= after_ms => {
            let since = last_input;
            (Presence::Away { since, locked: probe.locked }, Some(Turn::Left { since }))
        }
        Presence::Here => (p, None),
        Presence::Away { since, .. } if probe.locked => (Presence::Away { since, locked: true }, None),
        Presence::Away { since, locked: true } => (Presence::Here, Some(Turn::Back { since })),
        Presence::Away { since, locked: false } if probe.idle_ms < after_ms && last_input > since => {
            (Presence::Here, Some(Turn::Back { since }))
        }
        Presence::Away { .. } => (p, None),
    }
}

pub(crate) fn settings_path() -> PathBuf {
    crate::ingest::server::default_dir().join("settings.json")
}

pub(crate) fn read_settings(path: &Path) -> Value {
    std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null)
}

fn saved_minutes(path: &Path) -> u64 {
    read_settings(path)
        .get(SETTINGS_KEY)
        .and_then(Value::as_u64)
        .filter(|m| AWAY_AFTER_CHOICES.contains(m))
        .unwrap_or(AWAY_AFTER_MIN_DEFAULT)
}

fn saved_summary(path: &Path) -> bool {
    read_settings(path).get(SUMMARY_KEY).and_then(Value::as_bool).unwrap_or(false)
}

/// Read-modify-write of one key, keeping the others.
pub(crate) fn save_setting(path: &Path, key: &str, value: Value) -> std::io::Result<()> {
    let mut v = read_settings(path);
    if !v.is_object() {
        v = Value::Object(Default::default());
    }
    v[key] = value;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&v)?)?;
    std::fs::rename(&tmp, path)
}

fn env_override_ms() -> Option<u64> {
    std::env::var(ENV_SECS).ok()?.trim().parse::<u64>().ok().filter(|s| *s > 0).map(|s| s * 1000)
}

struct Inner {
    presence: Presence,
    minutes: u64,
    summary: bool,
    digest: Option<Digest>,
}

/// Owns presence and the card on screen; the activity log itself lives in the store.
pub struct Away {
    started: u64,
    inner: Mutex<Inner>,
    emit: Box<dyn Fn(Option<&Digest>) + Send + Sync>,
}

impl Away {
    pub fn new(emit: Box<dyn Fn(Option<&Digest>) + Send + Sync>) -> Arc<Away> {
        let path = settings_path();
        let away = Away::with(saved_minutes(&path), store::now_ms(), emit);
        away.inner.lock().unwrap().summary = saved_summary(&path);
        away
    }

    fn with(minutes: u64, started: u64, emit: Box<dyn Fn(Option<&Digest>) + Send + Sync>) -> Arc<Away> {
        let inner = Inner { presence: Presence::Here, minutes, summary: false, digest: None };
        Arc::new(Away { started, inner: Mutex::new(inner), emit })
    }

    pub fn summary_on(&self) -> bool {
        self.inner.lock().unwrap().summary
    }

    /// Puts model sentences on the card, if the card they were written for is still the one shown.
    pub fn fill(&self, since: u64, until: u64, sentences: &HashMap<String, String>) {
        let filled = {
            let mut inner = self.inner.lock().unwrap();
            let Some(d) = inner.digest.as_mut().filter(|d| d.since == since && d.until == until) else { return };
            let mut any = false;
            for e in &mut d.entries {
                if let Some(s) = sentences.get(&e.session_id) {
                    e.summary = Some(s.clone());
                    any = true;
                }
            }
            any.then(|| d.clone())
        };
        if let Some(d) = filled {
            (self.emit)(Some(&d));
        }
    }

    fn after_ms(&self) -> u64 {
        env_override_ms().unwrap_or(self.inner.lock().unwrap().minutes * 60_000)
    }

    pub fn current(&self) -> Option<Digest> {
        self.inner.lock().unwrap().digest.clone()
    }

    /// Esc, the close button, or any approval decision.
    pub fn dismiss(&self) {
        if self.inner.lock().unwrap().digest.take().is_some() {
            (self.emit)(None);
        }
    }

    /// Returns the card when this poll built a new one.
    pub fn poll(&self, store: &Store, probe: Probe, now: u64) -> Option<Digest> {
        let after = self.after_ms();
        let turn = {
            let mut inner = self.inner.lock().unwrap();
            let (next, turn) = step(inner.presence, probe, now, after);
            inner.presence = next;
            turn
        };
        match turn {
            Some(Turn::Back { since }) => {
                // Sessions found by the startup scan look new; the window opens once that has settled.
                let from = since.max(self.started + STARTUP_GRACE_MS);
                let d = store.digest(from, now).map(|d| Digest { since, ..d });
                println!("peepthis: back after {}s, digest {} entries", now.saturating_sub(since) / 1000, d.as_ref().map_or(0, |d| d.entries.len()));
                if let Some(d) = &d {
                    self.inner.lock().unwrap().digest = Some(d.clone());
                    (self.emit)(Some(d));
                }
                d
            }
            Some(Turn::Left { since }) => {
                println!("peepthis: away since {since}");
                None
            }
            None => None,
        }
    }

    pub fn spawn(self: &Arc<Self>, store: Arc<Store>) {
        let me = Arc::clone(self);
        let spawned = std::thread::Builder::new().name("away".into()).spawn(move || loop {
            std::thread::sleep(POLL);
            let Some(p) = probe() else { continue };
            if let Some(d) = me.poll(&store, p, store::now_ms()) {
                if me.summary_on() {
                    crate::summary::start(Arc::clone(&me), Arc::clone(&store), d);
                }
            }
        });
        if let Err(e) = spawned {
            eprintln!("peepthis: away thread failed: {e}");
        }
    }
}

#[cfg(windows)]
fn probe() -> Option<Probe> {
    use windows_sys::Win32::System::StationsAndDesktops::{CloseDesktop, OpenInputDesktop, DESKTOP_SWITCHDESKTOP};
    use windows_sys::Win32::System::SystemInformation::GetTickCount;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    // SAFETY: plain Win32 calls on a stack struct sized as the API requires.
    unsafe {
        if GetLastInputInfo(&mut info) == 0 {
            return None;
        }
        let idle_ms = GetTickCount().wrapping_sub(info.dwTime) as u64;
        // The lock screen is the secure desktop, which a normal process cannot open.
        let desk = OpenInputDesktop(0, 0, DESKTOP_SWITCHDESKTOP);
        let locked = desk.is_null();
        if !locked {
            CloseDesktop(desk);
        }
        Some(Probe { locked, idle_ms })
    }
}

#[cfg(not(windows))]
fn probe() -> Option<Probe> {
    None
}

pub fn wire(app: &tauri::App, store: &Arc<Store>) -> Arc<Away> {
    let handle = app.handle().clone();
    let away = Away::new(Box::new(move |d| {
        let _ = handle.emit("digest", d);
    }));
    away.spawn(Arc::clone(store));
    app.manage(Arc::clone(&away));
    away
}

#[tauri::command]
pub fn get_digest(away: tauri::State<'_, Arc<Away>>) -> Option<Digest> {
    away.current()
}

#[tauri::command]
pub fn dismiss_digest(away: tauri::State<'_, Arc<Away>>) {
    away.dismiss();
}

#[tauri::command]
pub fn away_after(away: tauri::State<'_, Arc<Away>>) -> u64 {
    away.inner.lock().unwrap().minutes
}

#[tauri::command]
pub fn set_away_after(minutes: u64, away: tauri::State<'_, Arc<Away>>) -> Result<u64, String> {
    if !AWAY_AFTER_CHOICES.contains(&minutes) {
        return Err(format!("{minutes} min is not one of the choices"));
    }
    save_setting(&settings_path(), SETTINGS_KEY, Value::from(minutes)).map_err(|e| e.to_string())?;
    away.inner.lock().unwrap().minutes = minutes;
    Ok(minutes)
}

#[tauri::command]
pub fn digest_summary(away: tauri::State<'_, Arc<Away>>) -> bool {
    away.summary_on()
}

#[tauri::command]
pub fn set_digest_summary(on: bool, away: tauri::State<'_, Arc<Away>>) -> Result<bool, String> {
    save_setting(&settings_path(), SUMMARY_KEY, Value::from(on)).map_err(|e| e.to_string())?;
    away.inner.lock().unwrap().summary = on;
    Ok(on)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{self, Event, Shell};

    const T0: u64 = 10_000_000;
    const MIN: u64 = 60_000;
    const AFTER: u64 = 5 * MIN;

    fn session(id: &str, state: State, since: u64) -> Session {
        let mut s = Session::new(id, 1, id, "C:/p", "interactive", "cli", T0 - 60 * MIN);
        s.state = state;
        s.state_since = since;
        s
    }

    fn here(idle_ms: u64) -> Probe {
        Probe { locked: false, idle_ms }
    }

    #[test]
    fn idle_past_the_threshold_leaves_from_the_last_input_and_input_comes_back() {
        let (p, t) = step(Presence::Here, here(AFTER - 1), T0, AFTER);
        assert_eq!((p, t), (Presence::Here, None));
        let (p, t) = step(p, here(AFTER), T0 + 1, AFTER);
        assert_eq!(t, Some(Turn::Left { since: T0 + 1 - AFTER }));
        let (p, t) = step(p, here(AFTER + 10 * MIN), T0 + 10 * MIN, AFTER);
        assert_eq!(t, None, "still idle");
        let (p, t) = step(p, here(200), T0 + 11 * MIN, AFTER);
        assert_eq!((p, t), (Presence::Here, Some(Turn::Back { since: T0 + 1 - AFTER })));
    }

    #[test]
    fn lock_leaves_at_once_and_unlock_comes_back() {
        let (p, t) = step(Presence::Here, Probe { locked: true, idle_ms: 2_000 }, T0, AFTER);
        assert_eq!(p, Presence::Away { since: T0 - 2_000, locked: true });
        assert_eq!(t, Some(Turn::Left { since: T0 - 2_000 }));
        // Input on the lock screen does not end the absence.
        let (p, t) = step(p, Probe { locked: true, idle_ms: 0 }, T0 + MIN, AFTER);
        assert_eq!(t, None);
        let (p, t) = step(p, here(30 * MIN), T0 + 30 * MIN, AFTER);
        assert_eq!((p, t), (Presence::Here, Some(Turn::Back { since: T0 - 2_000 })));
    }

    #[test]
    fn idle_away_that_then_locks_needs_the_unlock() {
        let (p, _) = step(Presence::Here, here(AFTER), T0, AFTER);
        let (p, _) = step(p, Probe { locked: true, idle_ms: AFTER + MIN }, T0 + MIN, AFTER);
        assert!(matches!(p, Presence::Away { locked: true, .. }));
        let (p, t) = step(p, here(100), T0 + 2 * MIN, AFTER);
        assert_eq!((p, t), (Presence::Here, Some(Turn::Back { since: T0 - AFTER })));
    }

    #[test]
    fn log_is_capped_and_marks_partial_counts() {
        let mut a = Activity::default();
        for i in 0..(NOTES_KEPT as u64 + 10) {
            a.note("s", "demo", Note::Command, T0 + i);
        }
        assert_eq!(a.len("s"), NOTES_KEPT);
        let sessions = [session("s", State::Working, T0 - MIN)];
        let d = digest(&a, &sessions, T0, T0 + 1_000).unwrap();
        assert_eq!(d.entries[0].commands, NOTES_KEPT as u32);
        assert!(d.entries[0].partial);
        let d = digest(&a, &sessions, T0 + 100, T0 + 1_000).unwrap();
        assert!(!d.entries[0].partial, "the drops were before the window");
    }

    #[test]
    fn log_keeps_paths_and_a_clipped_line_only() {
        let mut a = Activity::default();
        let long = "x".repeat(500);
        a.note("s", "demo", Note::Stop { line: Some(format!("\n  {long}\nsecond line")) }, T0);
        a.note("s", "demo", Note::File(format!("src/{long}.ts")), T0);
        let Some(log) = a.logs.get("s") else { panic!() };
        let Note::Stop { line: Some(line) } = &log.notes[0].1 else { panic!() };
        assert_eq!(line.chars().count(), LINE_MAX_CHARS);
        assert!(line.ends_with('…') && !line.contains("second"));
        let Note::File(p) = &log.notes[1].1 else { panic!() };
        assert_eq!(p.chars().count(), PATH_MAX_CHARS);
    }

    #[test]
    fn nothing_changed_means_no_card() {
        let mut a = Activity::default();
        a.note("s", "demo", Note::Command, T0 - MIN);
        let sessions = [session("s", State::Success, T0 - MIN), session("t", State::Idle, T0 - 2 * MIN)];
        assert_eq!(digest(&a, &sessions, T0, T0 + 10 * MIN), None);
        assert_eq!(digest(&Activity::default(), &[], T0, T0 + MIN), None);
    }

    #[test]
    fn groups_in_order_blocked_failed_finished_running() {
        let mut a = Activity::default();
        for id in ["run", "done", "err", "wait", "wait2"] {
            a.note(id, id, Note::Command, T0 + MIN);
        }
        let mut gone = session("stuck", State::Stuck, T0 + 3 * MIN);
        gone.task = "No response for 6 min".into();
        let sessions = [
            session("run", State::Working, T0 + 2 * MIN),
            session("done", State::Success, T0 + 4 * MIN),
            session("err", State::Error, T0 + 5 * MIN),
            session("wait", State::Waiting, T0 + 6 * MIN),
            session("wait2", State::Waiting, T0 + 2 * MIN),
            gone,
        ];
        let d = digest(&a, &sessions, T0, T0 + 10 * MIN).unwrap();
        let got: Vec<(&str, Group)> = d.entries.iter().map(|e| (e.name.as_str(), e.group)).collect();
        assert_eq!(
            got,
            vec![
                ("wait2", Group::Blocked),
                ("wait", Group::Blocked),
                ("err", Group::Failed),
                ("stuck", Group::Failed),
                ("done", Group::Finished),
                ("run", Group::Running),
            ]
        );
    }

    #[test]
    fn ended_and_removed_sessions_are_finished_and_a_failed_shell_counts_as_failed() {
        let mut a = Activity::default();
        a.note("gone", "old-name", Note::Stop { line: Some("All done.".into()) }, T0 + MIN);
        let mut dead = session("dead", State::Idle, T0 + 2 * MIN);
        dead.alive = false;
        dead.ended_at = Some(T0 + 2 * MIN);
        let mut shelly = session("shelly", State::Success, T0 - MIN);
        let mut sh = Shell::new("b1", "npm run build", "", T0);
        sh.state = ShellState::Failed;
        sh.ended_at = Some(T0 + MIN);
        shelly.shells.push(sh);
        a.note("shelly", "shelly", Note::Command, T0 + MIN);
        let d = digest(&a, &[dead, shelly], T0, T0 + 10 * MIN).unwrap();
        let got: Vec<(&str, Group, bool)> = d.entries.iter().map(|e| (e.name.as_str(), e.group, e.alive)).collect();
        assert_eq!(got, vec![("shelly", Group::Failed, true), ("dead", Group::Finished, false), ("old-name", Group::Finished, false)]);
        assert_eq!(d.entries[2].last_line.as_deref(), Some("All done."));
    }

    #[test]
    fn entry_counts_files_commands_releases_and_times_the_open_turn() {
        let mut a = Activity::default();
        a.note("s", "demo", Note::Prompt, T0 - 2 * MIN);
        a.note("s", "demo", Note::File("src/early.ts".into()), T0 - MIN);
        a.note("s", "demo", Note::File("src/a.ts".into()), T0 + MIN);
        a.note("s", "demo", Note::File("src/b.ts".into()), T0 + MIN);
        a.note("s", "demo", Note::File("src/a.ts".into()), T0 + 2 * MIN);
        a.note("s", "demo", Note::Command, T0 + 2 * MIN);
        a.note("s", "demo", Note::Command, T0 + 3 * MIN);
        a.note("s", "demo", Note::Released, T0 + 3 * MIN);
        a.note("s", "demo", Note::Stop { line: Some("Tests pass.".into()) }, T0 + 4 * MIN);
        let d = digest(&a, &[session("s", State::Success, T0 + 4 * MIN)], T0, T0 + 20 * MIN).unwrap();
        let e = &d.entries[0];
        assert_eq!((e.group, e.file_count, e.commands, e.released), (Group::Finished, 2, 2, 1));
        assert_eq!(e.files, vec!["src/b.ts".to_string(), "src/a.ts".to_string()]);
        assert_eq!(e.last_line.as_deref(), Some("Tests pass."));
        assert_eq!(e.took_ms, 6 * MIN, "from the prompt before leaving to the stop");
    }

    #[test]
    fn running_entry_times_to_now_and_lists_at_most_eight_files() {
        let mut a = Activity::default();
        a.note("s", "demo", Note::Stop { line: None }, T0 - 2 * MIN);
        for i in 0..12 {
            a.note("s", "demo", Note::File(format!("f{i}.rs")), T0 + MIN + i);
        }
        let d = digest(&a, &[session("s", State::Working, T0 + MIN)], T0, T0 + 9 * MIN).unwrap();
        let e = &d.entries[0];
        assert_eq!((e.file_count, e.files.len()), (12, FILES_LISTED));
        assert_eq!(e.took_ms, 8 * MIN, "the turn started inside the window");
    }

    #[test]
    fn a_state_change_alone_is_enough_and_pending_marks_inbox_entries() {
        let mut s = state::apply(session("s", State::Idle, T0 - MIN), Event::UserPromptSubmit, T0 - MIN);
        s = state::apply(s, Event::Notification { message: "Needs permission: Bash ls".into(), idle: false }, T0 + MIN);
        let d = digest(&Activity::default(), &[s], T0, T0 + 5 * MIN).unwrap();
        assert_eq!((d.entries[0].group, d.entries[0].pending), (Group::Blocked, false));
        assert_eq!(d.entries[0].state, Some(State::Waiting));
    }

    #[test]
    fn prune_keeps_live_sessions_and_recent_gone_ones() {
        let mut a = Activity::default();
        a.note("live", "", Note::Command, T0);
        a.note("gone-old", "", Note::Command, T0);
        a.note("gone-new", "", Note::Command, T0 + GONE_KEEP_MS);
        a.prune(|id| id == "live", T0 + GONE_KEEP_MS + 1);
        assert_eq!((a.len("live"), a.len("gone-old"), a.len("gone-new")), (1, 0, 1));
    }

    #[test]
    fn return_emits_the_card_once_and_dismiss_clears_it() {
        use crate::source::{SessionInfo, SourceEvent};
        let store = Store::new(Box::new(|_| {}));
        let info = SessionInfo { id: "s".into(), pid: 1, name: "demo".into(), title: None, cwd: "C:/p".into(), kind: "interactive".into(), entrypoint: "cli".into(), unparsed: false };
        store.ingest(SourceEvent::Upsert { info, busy: None }, T0);
        let seen = Arc::new(Mutex::new(Vec::<Option<usize>>::new()));
        let sink = Arc::clone(&seen);
        let away = Away::with(5, T0 - 60 * MIN, Box::new(move |d| sink.lock().unwrap().push(d.map(|d| d.entries.len()))));
        let poll = |idle: u64, now: u64| away.poll(&store, here(idle), now);

        poll(1_000, T0 + MIN);
        poll(AFTER, T0 + 6 * MIN);
        poll(100, T0 + 7 * MIN);
        assert!(seen.lock().unwrap().is_empty(), "nothing changed: no card");

        poll(AFTER, T0 + 12 * MIN);
        store.note("s", Note::Command, T0 + 13 * MIN);
        poll(AFTER + MIN, T0 + 13 * MIN);
        poll(50, T0 + 14 * MIN);
        assert_eq!(*seen.lock().unwrap(), vec![Some(1)]);
        let d = away.current().unwrap();
        assert_eq!((d.since, d.until), (T0 + 7 * MIN, T0 + 14 * MIN));
        away.dismiss();
        away.dismiss();
        assert_eq!(*seen.lock().unwrap(), vec![Some(1), None]);
        assert!(away.current().is_none());
    }

    #[test]
    fn fill_reaches_only_the_card_it_was_written_for() {
        let mut a = Activity::default();
        a.note("s", "demo", Note::Command, T0 + MIN);
        let card = digest(&a, &[session("s", State::Success, T0 + MIN)], T0, T0 + 5 * MIN).unwrap();
        let emits = Arc::new(Mutex::new(0));
        let sink = Arc::clone(&emits);
        let away = Away::with(5, T0, Box::new(move |_| *sink.lock().unwrap() += 1));
        let sentences = HashMap::from([("s".to_string(), "Ran the tests.".to_string()), ("x".to_string(), "Stray.".to_string())]);

        away.fill(T0, T0 + 5 * MIN, &sentences);
        assert_eq!(*emits.lock().unwrap(), 0, "no card yet");
        away.inner.lock().unwrap().digest = Some(card);
        away.fill(T0, T0 + 9 * MIN, &sentences);
        assert_eq!(*emits.lock().unwrap(), 0, "a newer card replaced it");
        away.fill(T0, T0 + 5 * MIN, &sentences);
        assert_eq!(*emits.lock().unwrap(), 1);
        assert_eq!(away.current().unwrap().entries[0].summary.as_deref(), Some("Ran the tests."));
    }

    #[test]
    fn the_startup_scan_is_not_news() {
        let store = Store::new(Box::new(|_| {}));
        store.apply_or_placeholder("s", Some("C:/p"), Event::UserPromptSubmit, T0 + 1_000);
        let away = Away::with(5, T0, Box::new(|_| {}));
        away.poll(&store, here(AFTER), T0 + 2_000);
        away.poll(&store, here(10), T0 + 3 * MIN);
        assert!(away.current().is_none());
    }

    #[test]
    fn minutes_setting_round_trips_and_rejects_odd_values() {
        let dir = std::env::temp_dir().join(format!("peepthis-away-{}", std::process::id()));
        let path = dir.join("settings.json");
        assert_eq!(saved_minutes(&path), AWAY_AFTER_MIN_DEFAULT);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, r#"{"trail":false,"awayAfterMin":7}"#).unwrap();
        assert_eq!(saved_minutes(&path), AWAY_AFTER_MIN_DEFAULT);
        save_setting(&path, SETTINGS_KEY, Value::from(10)).unwrap();
        assert_eq!(saved_minutes(&path), 10);
        assert!(!saved_summary(&path), "summary is off by default");
        save_setting(&path, SUMMARY_KEY, Value::Bool(true)).unwrap();
        assert!(saved_summary(&path));
        assert_eq!(read_settings(&path)["trail"], Value::Bool(false));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
