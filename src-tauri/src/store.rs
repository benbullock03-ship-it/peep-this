use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::source::{SessionInfo, SourceEvent};
use crate::state::{self, Event, Session};

pub const EMIT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimits {
    pub five_hour_pct: f64,
    pub seven_day_pct: f64,
    pub at: u64,
}

#[allow(dead_code)] // Installed/Outdated are set by the installer (AV-08).
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
}

struct Inner {
    sessions: HashMap<String, Session>,
    rate_limits: Option<RateLimits>,
    integration: Integration,
    dirty: bool,
}

impl Inner {
    fn snapshot(&self) -> Snapshot {
        let mut sessions: Vec<Session> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| a.id.cmp(&b.id));
        Snapshot { sessions, rate_limits: self.rate_limits, integration: self.integration }
    }
}

pub type Emit = Box<dyn Fn(&Snapshot) + Send + Sync>;

/// All session truth. Every mutation runs through `state::apply`; changes are pushed to `emit` at most once per `EMIT_INTERVAL`.
pub struct Store {
    inner: Mutex<Inner>,
    changed: Condvar,
}

impl Store {
    pub fn new(emit: Emit) -> Arc<Store> {
        let store = Arc::new(Store {
            inner: Mutex::new(Inner {
                sessions: HashMap::new(),
                rate_limits: None,
                integration: Integration::Missing,
                dirty: false,
            }),
            changed: Condvar::new(),
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

    fn mutate(&self, f: impl FnOnce(&mut Inner) -> bool) {
        let mut inner = self.inner.lock().unwrap();
        if f(&mut inner) {
            inner.dirty = true;
            self.changed.notify_one();
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.inner.lock().unwrap().snapshot()
    }

    pub fn ingest(&self, ev: SourceEvent, now: u64) {
        match ev {
            SourceEvent::Upsert { info, busy } => self.upsert(info, busy, now),
            SourceEvent::Ended { id } => self.apply(&id, Event::Ended, now),
        }
    }

    /// Feeds one event to a known session; events for unknown ids are dropped.
    pub fn apply(&self, id: &str, ev: Event, now: u64) {
        self.mutate(|inner| {
            let Some(s) = inner.sessions.get_mut(id) else { return false };
            let next = state::apply(s.clone(), ev, now);
            let changed = next != *s;
            *s = next;
            changed
        });
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
            inner.sessions.insert(info.id, s);
            changed
        });
    }
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
    fn events_for_unknown_sessions_are_dropped() {
        let (store, _) = recording_store();
        store.apply("ghost", Event::UserPromptSubmit, 1_000);
        store.ingest(SourceEvent::Ended { id: "ghost".into() }, 1_000);
        assert!(store.snapshot().sessions.is_empty());
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

        let snap = Snapshot {
            sessions: vec![],
            rate_limits: Some(RateLimits { five_hour_pct: 42.0, seven_day_pct: 18.0, at: 5 }),
            integration: Integration::Installed,
        };
        let v = serde_json::to_value(snap).unwrap();
        assert_eq!(v["rateLimits"]["fiveHourPct"], 42.0);
        assert_eq!(v["rateLimits"]["sevenDayPct"], 18.0);
        assert_eq!(v["integration"], "installed");
    }
}
