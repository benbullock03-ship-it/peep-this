use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::state::{self, Event, Session, State};
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
        if !s.alive || !matches!(s.state, State::Thinking | State::Working | State::Stuck) {
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

pub fn spawn(store: Arc<Store>) {
    std::thread::Builder::new()
        .name("ticker".into())
        .spawn(move || loop {
            std::thread::sleep(TICK_INTERVAL);
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
