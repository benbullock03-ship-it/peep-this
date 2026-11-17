use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::away;
use crate::closed;
use crate::learned;
use crate::limit;
use crate::mood::{self, Mood};
use crate::notify;
use crate::source::{SessionInfo, ShellQuery, SourceEvent};
use crate::state::{self, Event, Session};
use crate::ticker;
use crate::tickets::Repo;
use crate::tip::{self, Tip};

pub const EMIT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimits {
    pub five_hour_pct: f64,
    pub seven_day_pct: f64,
    /// Epoch ms, when the statusline payload carries it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_hour_resets_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seven_day_resets_at: Option<u64>,
    pub at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Integration {
    Installed,
    Missing,
    Outdated,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub sessions: Vec<Session>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<RateLimits>,
    pub integration: Integration,
    pub repos: Vec<Repo>,
    /// Ended sessions for the Closed shelf, newest first (AV-80).
    pub closed: Vec<closed::Entry>,
}

struct Inner {
    sessions: HashMap<String, Session>,
    rate_limits: Option<RateLimits>,
    integration: Integration,
    repos: Vec<Repo>,
    closed: Vec<closed::Entry>,
    dirty: bool,
    /// Sessions that entered `waiting` during the current mutation; drained outside the lock.
    entered: Vec<Session>,
    /// Sessions that ended during the current mutation, as they were just before; drained outside the lock.
    ended: Vec<closed::Ended>,
    /// Behind the away digest; never in the snapshot.
    activity: away::Activity,
    /// The widget's own headless sessions (digest summaries): never a row, never a hook.
    ignored: HashSet<String>,
    nudge_pct: f64,
    moods_on: bool,
    tips_seen: HashSet<Tip>,
    /// Where `tips_seen` persists; None in tests.
    tips_path: Option<PathBuf>,
    learned: learned::Learned,
    /// Where `learned` persists; None in tests.
    learned_path: Option<PathBuf>,
}

impl Inner {
    fn snapshot(&self) -> Snapshot {
        let mut sessions: Vec<Session> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| a.id.cmp(&b.id));
        Snapshot {
            sessions,
            rate_limits: self.rate_limits,
            integration: self.integration,
            repos: self.repos.clone(),
            closed: self.closed.clone(),
        }
    }

    fn note_end(&mut self, before: Option<&Session>, after: &Session, now: u64) {
        if let Some(b) = before.filter(|b| b.alive && !after.alive) {
            self.ended.push(closed::Ended::new(b, &self.repos, now));
        }
    }
}

pub type Emit = Box<dyn Fn(&Snapshot) + Send + Sync>;
pub type Alert = Box<dyn Fn(&Session) + Send + Sync>;
pub type OnEnd = Box<dyn Fn(closed::Ended) + Send + Sync>;
pub type OnLimits = Box<dyn Fn(&RateLimits) + Send + Sync>;

/// All session truth. Every mutation runs through `state::apply`; changes are pushed to `emit` at most once per `EMIT_INTERVAL`.
pub struct Store {
    inner: Mutex<Inner>,
    changed: Condvar,
    alert: Mutex<Option<Alert>>,
    on_end: Mutex<Option<OnEnd>>,
    on_limits: Mutex<Option<OnLimits>>,
}

impl Store {
    pub fn new(emit: Emit) -> Arc<Store> {
        let store = Arc::new(Store {
            inner: Mutex::new(Inner {
                sessions: HashMap::new(),
                rate_limits: None,
                integration: Integration::Missing,
                repos: Vec::new(),
                closed: Vec::new(),
                dirty: false,
                entered: Vec::new(),
                ended: Vec::new(),
                activity: away::Activity::default(),
                ignored: HashSet::new(),
                nudge_pct: mood::NUDGE_PCT_DEFAULT,
                moods_on: false,
                tips_seen: HashSet::new(),
                tips_path: None,
                learned: learned::Learned::default(),
                learned_path: None,
            }),
            changed: Condvar::new(),
            alert: Mutex::new(None),
            on_end: Mutex::new(None),
            on_limits: Mutex::new(None),
        });
        let worker = Arc::clone(&store);
        std::thread::Builder::new()
            .name("store-emit".into())
            .spawn(move || worker.emit_loop(emit))
            .expect("spawn store emitter");
        store
    }

    fn emit_loop(&self, emit: Emit) {
        loop {
            let snap = {
                let mut inner = self.inner.lock().unwrap();
                while !inner.dirty {
                    inner = self.changed.wait(inner).unwrap();
                }
                inner.dirty = false;
                inner.snapshot()
            };
            emit(&snap);
            std::thread::sleep(EMIT_INTERVAL);
        }
    }

    /// Called for each session that enters `waiting` or `error`, outside the session lock.
    pub fn set_alert(&self, alert: Alert) {
        *self.alert.lock().unwrap() = Some(alert);
    }

    /// Called with each session that ends, outside the session lock.
    pub fn set_on_end(&self, on_end: OnEnd) {
        *self.on_end.lock().unwrap() = Some(on_end);
    }

    /// Called with every rate-limit payload, outside the session lock.
    pub fn set_on_limits(&self, on_limits: OnLimits) {
        *self.on_limits.lock().unwrap() = Some(on_limits);
    }

    pub fn set_closed(&self, closed: Vec<closed::Entry>) {
        self.mutate(|inner| {
            let changed = inner.closed != closed;
            inner.closed = closed;
            changed
        });
    }

    fn mutate(&self, f: impl FnOnce(&mut Inner) -> bool) {
        let (entered, ended) = {
            let mut inner = self.inner.lock().unwrap();
            if f(&mut inner) | refresh_tips(&mut inner) {
                inner.dirty = true;
                self.changed.notify_one();
            }
            refresh_learned(&mut inner);
            (std::mem::take(&mut inner.entered), std::mem::take(&mut inner.ended))
        };
        if !entered.is_empty() {
            if let Some(alert) = self.alert.lock().unwrap().as_ref() {
                entered.iter().for_each(|s| alert(s));
            }
        }
        if !ended.is_empty() {
            if let Some(on_end) = self.on_end.lock().unwrap().as_ref() {
                ended.into_iter().for_each(on_end);
            }
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.inner.lock().unwrap().snapshot()
    }

    /// Drops every later event for `id`, and the session if it is already in.
    pub fn ignore(&self, id: &str) {
        self.mutate(|inner| {
            inner.ignored.insert(id.to_string());
            inner.sessions.remove(id).is_some()
        });
    }

    pub fn is_ignored(&self, id: &str) -> bool {
        self.inner.lock().unwrap().ignored.contains(id)
    }

    pub fn ingest(&self, ev: SourceEvent, now: u64) {
        let id = match &ev {
            SourceEvent::Upsert { info, .. } => &info.id,
            SourceEvent::Ended { id } | SourceEvent::Apply { id, .. } => id,
        };
        if self.is_ignored(id) {
            return;
        }
        match ev {
            SourceEvent::Upsert { info, busy } => self.upsert(info, busy, now),
            SourceEvent::Ended { id } => self.apply(&id, Event::Ended, now),
            SourceEvent::Apply { id, event } => self.apply(&id, event, now),
        }
    }

    /// Feeds one event to a known session; events for unknown ids are dropped.
    pub fn apply(&self, id: &str, ev: Event, now: u64) {
        self.mutate(|inner| {
            let Some(s) = inner.sessions.get_mut(id) else { return false };
            let mut next = state::apply(s.clone(), ev, now);
            refresh_branch(Some(&*s), &mut next);
            let changed = next != *s;
            let alert = notify::entered_alert(Some(&*s), &next).then(|| next.clone());
            let before = std::mem::replace(s, next.clone());
            inner.entered.extend(alert);
            inner.note_end(Some(&before), &next, now);
            refresh_moods(inner, now) || changed
        });
    }

    /// Like `apply`, but an unknown id gets a placeholder (pid 0, named after `cwd`) that a later session-file upsert fills in.
    pub fn apply_or_placeholder(&self, id: &str, cwd: Option<&str>, ev: Event, now: u64) {
        self.mutate(|inner| {
            if inner.ignored.contains(id) {
                return false;
            }
            let before = inner.sessions.get(id).cloned();
            let base = match before.clone() {
                // A resumed session reports SessionStart before its session file reappears.
                Some(s) if !s.alive && matches!(ev, Event::SessionStart) => {
                    Session::new(id, s.pid, s.name, s.cwd, s.kind, s.entrypoint, now)
                }
                Some(s) => s,
                None if matches!(ev, Event::Ended) => return false,
                None => placeholder(id, cwd.unwrap_or(""), now),
            };
            let mut next = state::apply(base, ev, now);
            refresh_branch(before.as_ref(), &mut next);
            let changed = before.as_ref() != Some(&next);
            if notify::entered_alert(before.as_ref(), &next) {
                inner.entered.push(next.clone());
            }
            inner.note_end(before.as_ref(), &next, now);
            inner.sessions.insert(id.to_string(), next);
            refresh_moods(inner, now) || changed
        });
    }

    pub fn tick(&self, now: u64) {
        self.mutate(|inner| {
            let changed = ticker::tick(&mut inner.sessions, now);
            let Inner { sessions, activity, .. } = inner;
            activity.prune(|id| sessions.contains_key(id), now);
            refresh_moods(inner, now) || changed
        });
    }

    /// Adds to a known session's activity log; unknown ids are dropped.
    pub fn note(&self, id: &str, note: away::Note, now: u64) {
        let mut inner = self.inner.lock().unwrap();
        let Some(name) = inner.sessions.get(id).map(|s| s.name.clone()) else { return };
        inner.activity.note(id, &name, note, now);
    }

    pub fn digest(&self, since: u64, now: u64) -> Option<away::Digest> {
        let inner = self.inner.lock().unwrap();
        let sessions: Vec<Session> = inner.sessions.values().cloned().collect();
        away::digest(&inner.activity, &sessions, since, now)
    }

    /// Shells of live sessions the probe should look at: running ones, plus ended ones whose output tail is unread.
    pub fn shell_queries(&self) -> Vec<ShellQuery> {
        let inner = self.inner.lock().unwrap();
        let mut out: Vec<ShellQuery> = inner
            .sessions
            .values()
            .filter(|s| s.alive)
            .flat_map(|s| {
                s.shells.iter().filter(|sh| sh.is_running() || !sh.tail_read).map(|sh| ShellQuery {
                    session_id: s.id.clone(),
                    session_pid: s.pid,
                    shell_id: sh.id.clone(),
                    command: sh.command.clone(),
                    pid: sh.pid,
                    ended: !sh.is_running(),
                })
            })
            .collect();
        out.sort_by(|a, b| a.session_id.cmp(&b.session_id));
        out
    }

    /// `resets` is (5h, 7d) in epoch ms.
    pub fn set_rate_limits(&self, five_hour_pct: f64, seven_day_pct: f64, resets: (Option<u64>, Option<u64>), now: u64) {
        let rl = RateLimits {
            five_hour_pct,
            seven_day_pct,
            five_hour_resets_at: resets.0,
            seven_day_resets_at: resets.1,
            at: now,
        };
        self.mutate(|inner| {
            let changed = inner.rate_limits.is_none_or(|r| RateLimits { at: now, ..r } != rl);
            inner.rate_limits = Some(rl);
            refresh_moods(inner, now) || changed
        });
        if let Some(on_limits) = self.on_limits.lock().unwrap().as_ref() {
            on_limits(&rl);
        }
    }

    /// The approval broker's held requests for one session; not routed through `state::apply`.
    pub fn set_pending(&self, id: &str, pending: Vec<crate::approve::PendingApproval>) {
        self.mutate(|inner| {
            let Some(s) = inner.sessions.get_mut(id) else { return false };
            if s.pending == pending {
                return false;
            }
            s.pending = pending;
            true
        });
    }

    pub fn set_integration(&self, integration: Integration) {
        self.mutate(|inner| {
            let changed = inner.integration != integration;
            inner.integration = integration;
            changed
        });
    }

    pub fn set_repos(&self, repos: Vec<Repo>, now: u64) {
        self.mutate(|inner| {
            let changed = inner.repos != repos;
            inner.repos = repos;
            refresh_moods(inner, now) || changed
        });
    }

    pub fn set_tips_seen(&self, seen: HashSet<Tip>, path: Option<PathBuf>) {
        self.mutate(|inner| {
            inner.tips_seen = seen;
            inner.tips_path = path;
            false
        });
    }

    pub fn set_learned(&self, l: learned::Learned, path: Option<PathBuf>) {
        self.mutate(|inner| {
            inner.learned = l;
            inner.learned_path = path;
            false
        });
    }

    /// The learned list after adding the patterns `cited` finds under the known repo roots (read outside the lock).
    pub fn learned_with(&self, cited: impl FnOnce(&[String]) -> Vec<String>) -> learned::Learned {
        let roots: Vec<String> = self.inner.lock().unwrap().repos.iter().map(|r| r.root.clone()).collect();
        let found = cited(&roots);
        let mut inner = self.inner.lock().unwrap();
        if learned::add(&mut inner.learned.patterns, found) {
            if let Some(path) = &inner.learned_path {
                learned::save(path, &inner.learned);
            }
        }
        inner.learned.clone()
    }

    pub fn dismiss_tip(&self, id: &str) {
        self.mutate(|inner| inner.sessions.get_mut(id).and_then(|s| s.tip.take()).is_some());
    }

    pub fn nudge_pct(&self) -> f64 {
        self.inner.lock().unwrap().nudge_pct
    }

    pub fn set_nudge_pct(&self, pct: f64, now: u64) {
        self.mutate(|inner| {
            inner.nudge_pct = pct;
            refresh_moods(inner, now)
        });
    }

    pub fn moods_on(&self) -> bool {
        self.inner.lock().unwrap().moods_on
    }

    pub fn set_moods_on(&self, on: bool, now: u64) {
        self.mutate(|inner| {
            inner.moods_on = on;
            refresh_moods(inner, now)
        });
    }

    /// (id, cwd) of live sessions, by id.
    pub fn live_cwds(&self) -> Vec<(String, String)> {
        let inner = self.inner.lock().unwrap();
        let mut out: Vec<(String, String)> =
            inner.sessions.values().filter(|s| s.alive).map(|s| (s.id.clone(), s.cwd.clone())).collect();
        out.sort();
        out
    }

    fn upsert(&self, info: SessionInfo, busy: Option<bool>, now: u64) {
        self.mutate(|inner| {
            let before = inner.sessions.get(&info.id).cloned();
            // A session id seen again after it ended (e.g. resumed) starts over.
            let mut s = match before.clone() {
                Some(s) if s.alive => s,
                _ => Session::new(&info.id, info.pid, "", "", "", "", now),
            };
            s.pid = info.pid;
            s.name = info.name;
            s.file_title = info.title;
            s.refresh_title();
            s.cwd = info.cwd;
            s.kind = info.kind;
            s.entrypoint = info.entrypoint;
            if let Some(busy) = busy {
                s = state::apply(s, Event::SessionFile { busy }, now);
            }
            refresh_branch(before.as_ref(), &mut s);
            let changed = before.as_ref() != Some(&s);
            if notify::entered_alert(before.as_ref(), &s) {
                inner.entered.push(s.clone());
            }
            inner.sessions.insert(info.id, s);
            refresh_moods(inner, now) || changed
        });
    }
}

/// Takes every raised signal: it becomes the tip unless one is on show or a once-ever tip was seen. True if a tip was set.
fn refresh_tips(inner: &mut Inner) -> bool {
    let mut shown = false;
    for s in inner.sessions.values_mut() {
        let Some(t) = s.signal.take() else { continue };
        if s.tip.is_some() || (tip::once_ever(t) && inner.tips_seen.contains(&t)) {
            continue;
        }
        s.tip = Some(t);
        shown = true;
        if tip::once_ever(t) {
            inner.tips_seen.insert(t);
            if let Some(path) = &inner.tips_path {
                tip::save(path, &inner.tips_seen);
            }
        }
    }
    shown
}

/// Records every tip on show and Peep skill running into the learned list, saving when one is new.
fn refresh_learned(inner: &mut Inner) {
    let tips: Vec<String> = inner.sessions.values().filter_map(|s| s.tip).map(learned::tip_id).collect();
    let skills: Vec<String> = inner.sessions.values().filter_map(|s| s.skill.clone()).collect();
    if learned::add(&mut inner.learned.tips, tips) | learned::add(&mut inner.learned.skills, skills) {
        if let Some(path) = &inner.learned_path {
            learned::save(path, &inner.learned);
        }
    }
}

/// Rederives every session's mood; true if any changed.
fn refresh_moods(inner: &mut Inner, now: u64) -> bool {
    let all: Vec<&Session> = inner.sessions.values().collect();
    let moods: Vec<(String, Option<Mood>, Option<String>)> = all
        .iter()
        .map(|s| {
            let m = mood::derive(s, &all, &inner.repos, inner.nudge_pct, limit::highest(inner.rate_limits.as_ref()), now);
            let m = mood::gate(m, inner.moods_on);
            let clash = (m == Some(Mood::Startled)).then(|| mood::clash(s, &all, now)).flatten();
            (s.id.clone(), m, clash)
        })
        .collect();
    let mut changed = false;
    for (id, m, clash) in moods {
        if let Some(s) = inner.sessions.get_mut(&id).filter(|s| s.mood != m || s.clash != clash) {
            s.mood = m;
            s.clash = clash;
            changed = true;
        }
    }
    changed
}

/// Reads `.git/HEAD` only when the session is new, revived, moved or changed state.
fn refresh_branch(before: Option<&Session>, next: &mut Session) {
    let due = before.is_none_or(|b| !b.alive || b.state != next.state || b.cwd != next.cwd || b.pid != next.pid);
    if due && next.alive {
        next.branch = crate::git::branch(&next.cwd);
    }
}

fn placeholder(id: &str, cwd: &str, now: u64) -> Session {
    let base = cwd.trim_end_matches(['/', '\\']).rsplit(['/', '\\']).next().unwrap_or("");
    let name = if base.is_empty() { id.chars().take(8).collect() } else { base.to_string() };
    Session::new(id, 0, name, cwd, "", "", now)
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use std::time::Instant;

    fn info(id: &str, name: &str) -> SessionInfo {
        SessionInfo {
            id: id.into(),
            pid: 100,
            name: name.into(),
            title: None,
            cwd: "C:/work/demo".into(),
            kind: "interactive".into(),
            entrypoint: "cli".into(),
        }
    }

    fn upsert(id: &str, busy: Option<bool>) -> SourceEvent {
        SourceEvent::Upsert { info: info(id, id), busy }
    }

    type Log = Arc<Mutex<Vec<(Instant, Snapshot)>>>;

    fn recording_store() -> (Arc<Store>, Log) {
        let log: Log = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&log);
        let store = Store::new(Box::new(move |s: &Snapshot| {
            sink.lock().unwrap().push((Instant::now(), s.clone()));
        }));
        (store, log)
    }

    fn settle() {
        std::thread::sleep(EMIT_INTERVAL * 3);
    }

    #[test]
    fn moods_refresh_live_on_changes_and_expire_on_tick() {
        let (store, _) = recording_store();
        let mood_of = |id: &str| store.snapshot().sessions.into_iter().find(|s| s.id == id).unwrap().mood;
        store.set_moods_on(true, 0);
        store.ingest(upsert("a", None), 1_000);
        store.ingest(upsert("b", None), 60_000);
        assert_eq!(mood_of("a"), Some(Mood::Greet));
        assert_eq!(mood_of("b"), None);
        store.tick(60_000 + mood::GREET_MS);
        assert_eq!(mood_of("a"), None);
        let closed = Some(crate::tickets::Closed { id: "AV-1".into(), at: 70_000 });
        let repo = Repo { root: "C:/work/demo".into(), sessions: vec!["a".into()], tickets: vec![], last_closed: closed, ..Default::default() };
        store.set_repos(vec![repo], 70_000);
        assert_eq!(mood_of("a"), Some(Mood::Cheer));
    }

    #[test]
    fn moods_off_by_default_leave_only_clash() {
        let (store, _) = recording_store();
        let mood_of = |id: &str| store.snapshot().sessions.into_iter().find(|s| s.id == id).unwrap().mood;
        let full = Event::Statusline { model: None, context_pct: Some(85.0), tokens: None, cache_ttl_ms: None };
        store.ingest(upsert("a", None), 1_000);
        store.ingest(upsert("b", None), 60_000);
        store.apply("b", full, 60_000);
        assert_eq!(mood_of("a"), None, "no greet");
        assert_eq!(mood_of("b"), None, "no yawn, so no context nudge either");
        store.set_moods_on(true, 60_001);
        assert_eq!(mood_of("a"), Some(Mood::Greet));
        assert_eq!(mood_of("b"), Some(Mood::Yawn));
        store.set_moods_on(false, 60_002);
        let edit = |p: &str| Event::Edited { path: p.into(), event: Box::new(Event::PostToolUse) };
        store.apply("a", edit("C:/work/demo/x.rs"), 61_000);
        store.apply("b", edit("C:/work/demo/x.rs"), 62_000);
        assert_eq!(mood_of("a"), Some(Mood::Startled));
        assert_eq!(mood_of("b"), Some(Mood::Startled));
        assert!(!store.moods_on());
    }

    #[test]
    fn tips_show_once_ever_except_crowded_and_never_over_another() {
        let (store, _) = recording_store();
        let tip_of = |id: &str| store.snapshot().sessions.into_iter().find(|s| s.id == id).unwrap().tip;
        let helper = || Event::SubagentStart { agent_id: "x".into(), agent_type: "Explore".into() };
        let crowded = Event::Statusline { model: None, context_pct: Some(70.0), tokens: Some(140_000), cache_ttl_ms: None };
        store.ingest(upsert("a", None), 1_000);
        store.ingest(upsert("b", None), 1_000);
        store.apply("a", helper(), 2_000);
        assert_eq!(tip_of("a"), Some(Tip::Helper));
        store.apply("a", crowded.clone(), 3_000);
        assert_eq!(tip_of("a"), Some(Tip::Helper), "one tip at a time; the dropped one is not seen");
        store.apply("b", helper(), 4_000);
        assert_eq!(tip_of("b"), None, "helper is once ever");
        store.apply("b", crowded, 5_000);
        assert_eq!(tip_of("b"), Some(Tip::Crowded), "crowded is once per session");
        store.dismiss_tip("b");
        assert_eq!(tip_of("b"), None);
        assert!(store.inner.lock().unwrap().sessions.values().all(|s| s.signal.is_none()), "every signal is taken");
    }

    #[test]
    fn learned_list_records_tips_shown_and_skills_run_and_cited_patterns() {
        let store = Store::new(Box::new(|_| {}));
        store.ingest(upsert("a", None), 1_000);
        store.apply("a", Event::SubagentStart { agent_id: "x".into(), agent_type: "Explore".into() }, 2_000);
        let skill = Event::Skill { skill: "ducks".into(), ticket: None, event: Box::new(Event::UserPromptSubmit) };
        store.apply("a", skill, 3_000);
        let l = store.learned_with(|_| vec!["god-file".into()]);
        assert_eq!(l.tips, ["helper".to_string()].into_iter().collect());
        assert_eq!(l.skills, ["ducks".to_string()].into_iter().collect());
        assert_eq!(l.patterns, ["god-file".to_string()].into_iter().collect());
    }

    #[test]
    fn cold_tip_fires_from_the_ticker_on_an_idle_session() {
        let store = Store::new(Box::new(|_| {}));
        store.ingest(upsert("a", None), 1_000);
        store.apply("a", Event::Statusline { model: None, context_pct: Some(5.0), tokens: Some(9_000), cache_ttl_ms: None }, 1_000);
        store.tick(1_000 + tip::CACHE_TTL_DEFAULT_MS);
        assert_eq!(store.snapshot().sessions[0].tip, Some(Tip::Cold));
    }

    #[test]
    fn nudge_setting_moves_the_yawn_live() {
        let (store, _) = recording_store();
        let mood_of = || store.snapshot().sessions[0].mood;
        store.set_moods_on(true, 0);
        store.ingest(upsert("a", None), 1_000);
        store.apply("a", Event::Statusline { model: None, context_pct: Some(72.0), tokens: None, cache_ttl_ms: None }, 2_000);
        assert_eq!(mood_of(), Some(Mood::Sweat));
        store.set_nudge_pct(70.0, 3_000);
        assert_eq!(mood_of(), Some(Mood::Yawn));
        store.set_nudge_pct(95.0, 4_000);
        assert_eq!(mood_of(), Some(Mood::Sweat));
        assert_eq!(store.nudge_pct(), 95.0);
    }

    #[test]
    fn upsert_creates_session_and_maps_busy() {
        let (store, _) = recording_store();
        store.ingest(upsert("a", Some(true)), 1_000);
        let snap = store.snapshot();
        assert_eq!(snap.sessions.len(), 1);
        let s = &snap.sessions[0];
        assert_eq!(s.name, "a");
        assert_eq!(s.cwd, "C:/work/demo");
        assert_eq!(s.state, State::Working);
        assert!(s.alive);
        assert_eq!(snap.integration, Integration::Missing);
    }

    #[test]
    fn title_prefers_user_name_then_custom_then_ai_title() {
        let (store, _) = recording_store();
        let title = || store.snapshot().sessions[0].title.clone();
        store.ingest(upsert("a", None), 1_000);
        assert_eq!(title(), None);
        store.apply("a", Event::TranscriptTitle { title: "AI guess".into(), user: false }, 2_000);
        assert_eq!(title().as_deref(), Some("AI guess"));
        store.apply("a", Event::TranscriptTitle { title: "Renamed".into(), user: true }, 3_000);
        assert_eq!(title().as_deref(), Some("Renamed"));
        let mut named = info("a", "From file");
        named.title = Some("From file".into());
        store.ingest(SourceEvent::Upsert { info: named, busy: None }, 4_000);
        assert_eq!(title().as_deref(), Some("From file"));
        store.ingest(upsert("a", None), 5_000);
        assert_eq!(title().as_deref(), Some("Renamed"));
    }

    #[test]
    fn branch_is_read_on_sight_and_refreshed_on_state_change() {
        let repo = std::env::temp_dir().join(format!("peepthis-store-branch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&repo);
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        let (store, _) = recording_store();
        let mut i = info("a", "a");
        i.cwd = repo.to_string_lossy().into_owned();
        store.ingest(SourceEvent::Upsert { info: i, busy: None }, 1_000);
        let branch = || store.snapshot().sessions[0].branch.clone();
        assert_eq!(branch().as_deref(), Some("main"));
        std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/feature\n").unwrap();
        store.apply("a", Event::TranscriptModel { model: "Opus".into() }, 2_000);
        assert_eq!(branch().as_deref(), Some("main"), "no state change, no read");
        store.apply("a", Event::UserPromptSubmit, 3_000);
        assert_eq!(branch().as_deref(), Some("feature"));
        std::fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn upsert_without_status_stays_idle_and_updates_metadata() {
        let (store, _) = recording_store();
        store.ingest(upsert("a", None), 1_000);
        store.ingest(SourceEvent::Upsert { info: info("a", "renamed"), busy: None }, 2_000);
        let s = &store.snapshot().sessions[0];
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.name, "renamed");
        assert_eq!(s.state_since, 1_000);
    }

    #[test]
    fn ended_marks_session_dead_and_resume_restarts_it() {
        let (store, _) = recording_store();
        store.ingest(upsert("a", Some(true)), 1_000);
        store.ingest(SourceEvent::Ended { id: "a".into() }, 2_000);
        let s = store.snapshot().sessions[0].clone();
        assert!(!s.alive);
        assert_eq!(s.ended_at, Some(2_000));
        assert_eq!(s.state, State::Idle);

        store.ingest(upsert("a", None), 3_000);
        let s = store.snapshot().sessions[0].clone();
        assert!(s.alive);
        assert_eq!(s.ended_at, None);
    }

    #[test]
    fn killed_session_dims_then_leaves_after_a_minute_via_tick() {
        let (store, log) = recording_store();
        store.ingest(upsert("a", Some(false)), 1_000);
        store.apply("a", Event::UserPromptSubmit, 1_000);
        store.tick(1_000 + state::STUCK_AFTER_MS);
        assert_eq!(store.snapshot().sessions[0].state, State::Stuck);

        store.ingest(SourceEvent::Ended { id: "a".into() }, 400_000);
        store.tick(400_000 + ticker::ENDED_LINGER_MS - 1);
        let s = store.snapshot().sessions[0].clone();
        assert!(!s.alive);
        assert_eq!(s.state, State::Idle);
        settle();
        let emits = log.lock().unwrap().len();
        store.tick(400_000 + ticker::ENDED_LINGER_MS);
        assert!(store.snapshot().sessions.is_empty());
        settle();
        assert_eq!(log.lock().unwrap().len(), emits + 1);
    }

    #[test]
    fn ending_hands_over_the_session_as_it_was_and_closed_reaches_the_snapshot() {
        let (store, _) = recording_store();
        let got = Arc::new(Mutex::new(Vec::<closed::Ended>::new()));
        let sink = Arc::clone(&got);
        store.set_on_end(Box::new(move |e| sink.lock().unwrap().push(e)));
        store.ingest(upsert("a", Some(false)), 1_000);
        store.apply("a", Event::Notification { message: "Claude needs your permission".into(), idle: false }, 2_000);
        store.ingest(SourceEvent::Ended { id: "a".into() }, 3_000);
        store.apply_or_placeholder("a", None, Event::Ended, 3_500);
        let got = got.lock().unwrap();
        assert_eq!(got.len(), 1, "a second Ended for a dead session is not another end");
        assert_eq!((got[0].session.state, got[0].at), (State::Waiting, 3_000));

        let entry = closed::entry(&got[0]);
        store.set_closed(vec![entry.clone()]);
        let v = serde_json::to_value(store.snapshot()).unwrap();
        assert_eq!(v["closed"][0]["sessionId"], "a");
        assert_eq!(v["closed"][0]["state"], "waiting");
        assert_eq!(v["closed"][0]["flags"]["attention"], true);
    }

    #[test]
    fn events_for_unknown_sessions_are_dropped() {
        let (store, _) = recording_store();
        store.apply("ghost", Event::UserPromptSubmit, 1_000);
        store.ingest(SourceEvent::Ended { id: "ghost".into() }, 1_000);
        assert!(store.snapshot().sessions.is_empty());
    }

    #[test]
    fn hook_for_unknown_session_creates_placeholder_that_upsert_fills() {
        let (store, _) = recording_store();
        store.apply_or_placeholder("p", Some("C:\\work\\demo\\"), Event::UserPromptSubmit, 1_000);
        let s = store.snapshot().sessions[0].clone();
        assert_eq!((s.pid, s.name.as_str(), s.cwd.as_str()), (0, "demo", "C:\\work\\demo\\"));
        assert_eq!(s.state, State::Thinking);
        assert!(s.alive);

        store.ingest(SourceEvent::Upsert { info: info("p", "real-name"), busy: Some(true) }, 2_000);
        let s = store.snapshot().sessions[0].clone();
        assert_eq!((s.pid, s.name.as_str(), s.kind.as_str()), (100, "real-name", "interactive"));
        assert_eq!(s.state, State::Thinking, "fresh hook state must survive the session file");
        assert_eq!(s.state_since, 1_000);
    }

    #[test]
    fn placeholder_without_cwd_is_named_by_id_prefix() {
        let (store, _) = recording_store();
        store.apply_or_placeholder("abcdef0123456789", None, Event::Stop, 1_000);
        assert_eq!(store.snapshot().sessions[0].name, "abcdef01");
    }

    #[test]
    fn ended_for_unknown_session_creates_nothing() {
        let (store, _) = recording_store();
        store.apply_or_placeholder("ghost", Some("C:/x"), Event::Ended, 1_000);
        assert!(store.snapshot().sessions.is_empty());
    }

    #[test]
    fn session_start_revives_ended_session_keeping_identity() {
        let (store, _) = recording_store();
        store.ingest(upsert("a", None), 1_000);
        store.apply_or_placeholder("a", None, Event::Ended, 2_000);
        store.apply_or_placeholder("a", None, Event::UserPromptSubmit, 3_000);
        assert!(!store.snapshot().sessions[0].alive);
        store.apply_or_placeholder("a", None, Event::SessionStart, 4_000);
        let s = store.snapshot().sessions[0].clone();
        assert!(s.alive);
        assert_eq!((s.pid, s.name.as_str()), (100, "a"));
    }

    #[test]
    fn rate_limits_set_and_emit_only_on_change() {
        let (store, log) = recording_store();
        store.set_rate_limits(42.0, 18.0, (Some(9_000), None), 1_000);
        settle();
        store.set_rate_limits(42.0, 18.0, (Some(9_000), None), 2_000);
        settle();
        assert_eq!(log.lock().unwrap().len(), 1);
        assert_eq!(
            store.snapshot().rate_limits,
            Some(RateLimits {
                five_hour_pct: 42.0,
                seven_day_pct: 18.0,
                five_hour_resets_at: Some(9_000),
                seven_day_resets_at: None,
                at: 2_000
            })
        );
        store.set_rate_limits(43.0, 18.0, (Some(9_000), None), 3_000);
        settle();
        assert_eq!(log.lock().unwrap().len(), 2);
    }

    #[test]
    fn a_limit_at_80_sweats_every_live_session_and_reports_each_payload() {
        let (store, _) = recording_store();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        store.set_on_limits(Box::new(move |rl| sink.lock().unwrap().push(rl.seven_day_pct)));
        store.set_moods_on(true, 0);
        store.ingest(upsert("a", None), 1_000);
        store.ingest(upsert("b", None), 1_000);
        store.set_rate_limits(10.0, 79.0, (None, None), 1_000);
        assert!(store.snapshot().sessions.iter().all(|s| s.mood.is_none()));
        store.set_rate_limits(10.0, 80.0, (None, None), 1_001);
        assert!(store.snapshot().sessions.iter().all(|s| s.mood == Some(Mood::Sweat)));
        assert_eq!(*seen.lock().unwrap(), vec![79.0, 80.0]);
    }

    #[test]
    fn integration_emits_only_on_change() {
        let (store, log) = recording_store();
        store.set_integration(Integration::Missing);
        settle();
        assert!(log.lock().unwrap().is_empty());
        store.set_integration(Integration::Installed);
        settle();
        assert_eq!(log.lock().unwrap().len(), 1);
        assert_eq!(store.snapshot().integration, Integration::Installed);
    }

    #[test]
    fn alert_fires_once_on_entering_waiting() {
        let (store, _) = recording_store();
        let alerts = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let sink = Arc::clone(&alerts);
        store.set_alert(Box::new(move |s: &Session| sink.lock().unwrap().push((s.name.clone(), s.task.clone()))));
        let perm = || Event::Notification { message: "Claude needs your permission to use Bash".into(), idle: false };
        store.ingest(upsert("a", Some(false)), 1_000);
        store.apply("a", Event::UserPromptSubmit, 2_000);
        store.apply("a", perm(), 3_000);
        store.apply("a", perm(), 4_000);
        store.apply("a", Event::PostToolUse, 5_000);
        store.apply_or_placeholder("a", None, perm(), 6_000);
        assert_eq!(
            *alerts.lock().unwrap(),
            vec![("a".to_string(), "Claude needs your permission to use Bash".to_string()); 2]
        );
    }

    #[test]
    fn hook_events_route_through_state_engine() {
        let (store, _) = recording_store();
        store.ingest(upsert("a", Some(false)), 1_000);
        store.apply("a", Event::UserPromptSubmit, 2_000);
        assert_eq!(store.snapshot().sessions[0].state, State::Thinking);
    }

    #[test]
    fn emits_once_per_change_and_not_for_no_ops() {
        let (store, log) = recording_store();
        store.ingest(upsert("a", Some(true)), 1_000);
        settle();
        assert_eq!(log.lock().unwrap().len(), 1);
        for t in 0..5 {
            store.ingest(upsert("a", Some(true)), 1_100 + t);
        }
        settle();
        assert_eq!(log.lock().unwrap().len(), 1, "identical rescans must not emit");
        store.ingest(upsert("b", None), 2_000);
        settle();
        let log = log.lock().unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[1].1.sessions.len(), 2);
    }

    #[test]
    fn bursts_are_throttled_and_last_state_wins() {
        let (store, log) = recording_store();
        let start = Instant::now();
        for i in 0..200 {
            store.ingest(upsert(&format!("s{i:03}"), None), 1_000);
        }
        std::thread::sleep(Duration::from_millis(50));
        for i in 200..250 {
            store.ingest(upsert(&format!("s{i:03}"), None), 1_000);
        }
        settle();
        let log = log.lock().unwrap();
        let elapsed = start.elapsed();
        let max = (elapsed.as_millis() / EMIT_INTERVAL.as_millis()) as usize + 1;
        assert!(log.len() <= max, "{} emits in {elapsed:?}", log.len());
        for pair in log.windows(2) {
            assert!(pair[1].0 - pair[0].0 >= EMIT_INTERVAL - Duration::from_millis(5));
        }
        assert_eq!(log.last().unwrap().1.sessions.len(), 250);
    }

    #[test]
    fn snapshot_serialises_to_spec_shape() {
        let (store, _) = recording_store();
        store.ingest(upsert("a", None), 1_000);
        let v = serde_json::to_value(store.snapshot()).unwrap();
        assert_eq!(v["integration"], "missing");
        assert!(v.get("rateLimits").is_none());
        assert_eq!(v["sessions"][0]["id"], "a");
        assert_eq!(v["sessions"][0]["state"], "idle");
        assert_eq!(v["repos"], serde_json::json!([]));

        let snap = Snapshot {
            sessions: vec![],
            rate_limits: Some(RateLimits {
                five_hour_pct: 42.0,
                seven_day_pct: 18.0,
                five_hour_resets_at: Some(7),
                seven_day_resets_at: None,
                at: 5,
            }),
            integration: Integration::Installed,
            repos: vec![],
            closed: vec![],
        };
        let v = serde_json::to_value(snap).unwrap();
        assert_eq!(v["rateLimits"]["fiveHourPct"], 42.0);
        assert_eq!(v["rateLimits"]["sevenDayPct"], 18.0);
        assert_eq!(v["rateLimits"]["fiveHourResetsAt"], 7);
        assert!(v["rateLimits"].get("sevenDayResetsAt").is_none());
        assert_eq!(v["integration"], "installed");
    }
}
