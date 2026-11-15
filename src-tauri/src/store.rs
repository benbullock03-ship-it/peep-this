use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::away;
use crate::notify;
use crate::source::{SessionInfo, ShellQuery, SourceEvent};
use crate::state::{self, Event, Session};
use crate::ticker;
use crate::tickets::Repo;

pub const EMIT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimits {
    pub five_hour_pct: f64,
    pub seven_day_pct: f64,
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
}

struct Inner {
    sessions: HashMap<String, Session>,
    rate_limits: Option<RateLimits>,
    integration: Integration,
    repos: Vec<Repo>,
    dirty: bool,
    /// Sessions that entered `waiting` during the current mutation; drained outside the lock.
    entered: Vec<Session>,
    /// Behind the away digest; never in the snapshot.
    activity: away::Activity,
    /// The widget's own headless sessions (digest summaries): never a row, never a hook.
    ignored: HashSet<String>,
}

impl Inner {
    fn snapshot(&self) -> Snapshot {
        let mut sessions: Vec<Session> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| a.id.cmp(&b.id));
        Snapshot { sessions, rate_limits: self.rate_limits, integration: self.integration, repos: self.repos.clone() }
    }
}

pub type Emit = Box<dyn Fn(&Snapshot) + Send + Sync>;
pub type Alert = Box<dyn Fn(&Session) + Send + Sync>;

/// All session truth. Every mutation runs through `state::apply`; changes are pushed to `emit` at most once per `EMIT_INTERVAL`.
pub struct Store {
    inner: Mutex<Inner>,
    changed: Condvar,
    alert: Mutex<Option<Alert>>,
}

impl Store {
    pub fn new(emit: Emit) -> Arc<Store> {
        let store = Arc::new(Store {
            inner: Mutex::new(Inner {
                sessions: HashMap::new(),
                rate_limits: None,
                integration: Integration::Missing,
                repos: Vec::new(),
                dirty: false,
                entered: Vec::new(),
                activity: away::Activity::default(),
                ignored: HashSet::new(),
            }),
            changed: Condvar::new(),
            alert: Mutex::new(None),
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

    /// Called for each session that enters `waiting`, outside the session lock.
    pub fn set_alert(&self, alert: Alert) {
        *self.alert.lock().unwrap() = Some(alert);
    }

    fn mutate(&self, f: impl FnOnce(&mut Inner) -> bool) {
        let entered = {
            let mut inner = self.inner.lock().unwrap();
            if f(&mut inner) {
                inner.dirty = true;
                self.changed.notify_one();
            }
            std::mem::take(&mut inner.entered)
        };
        if entered.is_empty() {
            return;
        }
        if let Some(alert) = self.alert.lock().unwrap().as_ref() {
            entered.iter().for_each(|s| alert(s));
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
            let next = state::apply(s.clone(), ev, now);
            let changed = next != *s;
            let alert = notify::entered_waiting(Some(&*s), &next).then(|| next.clone());
            *s = next;
            inner.entered.extend(alert);
            changed
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
            let next = state::apply(base, ev, now);
            let changed = before.as_ref() != Some(&next);
            if notify::entered_waiting(before.as_ref(), &next) {
                inner.entered.push(next.clone());
            }
            inner.sessions.insert(id.to_string(), next);
            changed
        });
    }

    pub fn tick(&self, now: u64) {
        self.mutate(|inner| {
            let changed = ticker::tick(&mut inner.sessions, now);
            let Inner { sessions, activity, .. } = inner;
            activity.prune(|id| sessions.contains_key(id), now);
            changed
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

    pub fn set_rate_limits(&self, five_hour_pct: f64, seven_day_pct: f64, now: u64) {
        self.mutate(|inner| {
            let changed = inner
                .rate_limits
                .is_none_or(|r| r.five_hour_pct != five_hour_pct || r.seven_day_pct != seven_day_pct);
            inner.rate_limits = Some(RateLimits { five_hour_pct, seven_day_pct, at: now });
            changed
        });
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

    pub fn set_repos(&self, repos: Vec<Repo>) {
        self.mutate(|inner| {
            let changed = inner.repos != repos;
            inner.repos = repos;
            changed
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
            s.cwd = info.cwd;
            s.kind = info.kind;
            s.entrypoint = info.entrypoint;
            if let Some(busy) = busy {
                s = state::apply(s, Event::SessionFile { busy }, now);
            }
            let changed = before.as_ref() != Some(&s);
            if notify::entered_waiting(before.as_ref(), &s) {
                inner.entered.push(s.clone());
            }
            inner.sessions.insert(info.id, s);
            changed
        });
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
        store.set_rate_limits(42.0, 18.0, 1_000);
        settle();
        store.set_rate_limits(42.0, 18.0, 2_000);
        settle();
        assert_eq!(log.lock().unwrap().len(), 1);
        assert_eq!(
            store.snapshot().rate_limits,
            Some(RateLimits { five_hour_pct: 42.0, seven_day_pct: 18.0, at: 2_000 })
        );
        store.set_rate_limits(43.0, 18.0, 3_000);
        settle();
        assert_eq!(log.lock().unwrap().len(), 2);
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
            rate_limits: Some(RateLimits { five_hour_pct: 42.0, seven_day_pct: 18.0, at: 5 }),
            integration: Integration::Installed,
            repos: vec![],
        };
        let v = serde_json::to_value(snap).unwrap();
        assert_eq!(v["rateLimits"]["fiveHourPct"], 42.0);
        assert_eq!(v["rateLimits"]["sevenDayPct"], 18.0);
        assert_eq!(v["integration"], "installed");
    }
}
