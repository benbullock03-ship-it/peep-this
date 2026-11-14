use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::source::ShellQuery;
use crate::state::{self, Event, Session, ShellObservation, State};
use crate::store::{self, Store};

pub const TICK_INTERVAL: Duration = Duration::from_secs(5);
pub const ENDED_LINGER_MS: u64 = 60_000;

/// One ticker pass: feeds `Tick` to busy sessions and drops sessions ended at least `ENDED_LINGER_MS` ago. Returns whether anything changed.
pub fn tick(sessions: &mut HashMap<String, Session>, now: u64) -> bool {
    let before = sessions.len();
    sessions.retain(|_, s| s.ended_at.is_none_or(|t| now.saturating_sub(t) < ENDED_LINGER_MS));
    let mut changed = sessions.len() != before;
    for s in sessions.values_mut() {
        // Stuck is ticked too so its "No response for N min" keeps counting.
        let busy = matches!(s.state, State::Thinking | State::Working | State::Stuck) || s.agents.iter().any(|a| a.is_busy());
        if !s.alive || !busy {
            continue;
        }
        let next = state::apply(s.clone(), Event::Tick, now);
        if next != *s {
            *s = next;
            changed = true;
        }
    }
    changed
}

/// OS lookups for background shells; returns one observation per query, in order.
pub type ShellProbe = Box<dyn Fn(&[ShellQuery]) -> Vec<ShellObservation> + Send>;

/// Runs `probe` outside the store lock and feeds each result back as `ShellProbe` events.
pub fn probe_shells(store: &Store, probe: &ShellProbe, now: u64) {
    let queries = store.shell_queries();
    if queries.is_empty() {
        return;
    }
    for (q, obs) in queries.iter().zip(probe(&queries)) {
        store.apply(&q.session_id, Event::ShellProbe { id: q.shell_id.clone(), obs }, now);
    }
}

pub fn spawn(store: Arc<Store>, probe: ShellProbe) {
    std::thread::Builder::new()
        .name("ticker".into())
        .spawn(move || loop {
            std::thread::sleep(TICK_INTERVAL);
            probe_shells(&store, &probe, store::now_ms());
            store.tick(store::now_ms());
        })
        .expect("spawn ticker");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::STUCK_AFTER_MS;

    const T0: u64 = 1_000_000;

    fn map(sessions: Vec<Session>) -> HashMap<String, Session> {
        sessions.into_iter().map(|s| (s.id.clone(), s)).collect()
    }

    fn session(id: &str, events: Vec<Event>) -> Session {
        let base = Session::new(id, 1, id, "C:/p", "interactive", "cli", T0);
        events.into_iter().fold(base, |s, e| state::apply(s, e, T0))
    }

    #[test]
    fn ticker_moves_quiet_thinking_session_to_stuck_at_6_min() {
        let mut m = map(vec![session("a", vec![Event::UserPromptSubmit])]);
        assert!(!tick(&mut m, T0 + STUCK_AFTER_MS - 1));
        assert_eq!(m["a"].state, State::Thinking);
        assert!(tick(&mut m, T0 + STUCK_AFTER_MS));
        assert_eq!(m["a"].state, State::Stuck);
        assert_eq!(m["a"].task, "No response for 6 min");
        assert!(tick(&mut m, T0 + STUCK_AFTER_MS + 60_000));
        assert_eq!(m["a"].task, "No response for 7 min");
    }

    #[test]
    fn ticker_leaves_non_busy_sessions_alone() {
        let mut m = map(vec![
            session("idle", vec![]),
            session("done", vec![Event::Stop]),
            session("wait", vec![Event::Notification { message: "perm".into(), idle: false }]),
        ]);
        let before = m.clone();
        assert!(!tick(&mut m, T0 + 60 * STUCK_AFTER_MS));
        assert_eq!(m, before);
    }

    #[test]
    fn probe_seam_feeds_observations_for_running_shells_of_live_sessions_only() {
        use crate::state::{ShellEvent, ShellExit, ShellState};
        use std::sync::Mutex;
        let shell = |id: &str| Event::Shell {
            update: ShellEvent::Started { id: id.into(), command: format!("cmd {id}"), description: String::new() },
            event: Box::new(Event::PostToolUse),
        };
        let store = Store::new(Box::new(|_| {}));
        store.apply_or_placeholder("live", None, shell("b1"), T0);
        store.apply_or_placeholder("live", None, shell("b2"), T0);
        store.apply_or_placeholder(
            "live",
            None,
            Event::Shell { update: ShellEvent::Ended { id: "b2".into(), exit: ShellExit::Code(1) }, event: Box::new(Event::UserPromptSubmit) },
            T0,
        );
        store.apply_or_placeholder("dead", None, shell("b9"), T0);
        store.apply_or_placeholder("dead", None, Event::Ended, T0);

        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let probe: ShellProbe = Box::new(move |qs| {
            sink.lock().unwrap().extend(qs.iter().map(|q| (q.shell_id.clone(), q.command.clone(), q.ended)));
            qs.iter()
                .map(|q| ShellObservation { port: (!q.ended).then_some(8080), last_line: Some("boom".into()), ..Default::default() })
                .collect()
        });
        probe_shells(&store, &probe, T0 + 5_000);
        assert_eq!(*seen.lock().unwrap(), vec![("b1".into(), "cmd b1".into(), false), ("b2".into(), "cmd b2".into(), true)]);
        let s = store.snapshot().sessions.into_iter().find(|s| s.id == "live").unwrap();
        assert_eq!((s.shells[0].state, s.shells[0].port), (ShellState::Serving, Some(8080)));
        assert_eq!((s.shells[1].state, s.shells[1].last_line.as_deref()), (ShellState::Failed, Some("boom")));

        seen.lock().unwrap().clear();
        probe_shells(&store, &probe, T0 + 10_000);
        assert_eq!(seen.lock().unwrap().len(), 1, "an ended shell's tail is read once");
    }

    #[test]
    fn ticker_removes_ended_session_60s_after_ended_at() {
        let ended = state::apply(session("a", vec![Event::Stop]), Event::Ended, T0 + 1_000);
        let mut m = map(vec![ended, session("b", vec![])]);
        assert!(!tick(&mut m, T0 + 1_000 + ENDED_LINGER_MS - 1));
        assert_eq!(m["a"].state, State::Success);
        assert!(tick(&mut m, T0 + 1_000 + ENDED_LINGER_MS));
        assert!(!m.contains_key("a"));
        assert!(m.contains_key("b"));
    }
}
