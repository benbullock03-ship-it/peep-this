//! AV-41: hover peek (spec §12.6). A pure machine fed by a cursor poll in window.rs.

/// Hover this long over the strip before the panel unfolds as a peek.
pub const OPEN_MS: u64 = 200;
/// Away from the peek and the strip's spot this long before it folds back.
pub const CLOSE_MS: u64 = 300;
pub const POLL_MS: u64 = 25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Away,
    /// Over the strip, but it may not arm until the cursor has left once: it arrived with a
    /// button down, a drag, a fold, or from the panel collapsing under it.
    Resting,
    Arming { since: u64 },
    Open,
    Closing { since: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input {
    pub now: u64,
    /// The plain strip is showing (not hidden, not a peek, not the panel).
    pub strip: bool,
    pub peeking: bool,
    /// Over the window, or during a peek over the strip's own spot too.
    pub over: bool,
    /// A mouse button is down, a drag is under way, or a switch is animating.
    pub blocked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Open,
    Close,
}

pub fn step(phase: Phase, i: Input) -> (Phase, Action) {
    if i.peeking {
        return match phase {
            _ if i.over || i.blocked => (Phase::Open, Action::None),
            Phase::Closing { since } if i.now.saturating_sub(since) >= CLOSE_MS => (Phase::Resting, Action::Close),
            Phase::Closing { since } => (Phase::Closing { since }, Action::None),
            _ => (Phase::Closing { since: i.now }, Action::None),
        };
    }
    if !i.strip {
        return (Phase::Resting, Action::None);
    }
    if !i.over {
        return (Phase::Away, Action::None);
    }
    if i.blocked {
        return (Phase::Resting, Action::None);
    }
    match phase {
        Phase::Away => (Phase::Arming { since: i.now }, Action::None),
        Phase::Arming { since } if i.now.saturating_sub(since) >= OPEN_MS => (Phase::Open, Action::Open),
        Phase::Arming { since } => (Phase::Arming { since }, Action::None),
        _ => (Phase::Resting, Action::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(now: u64, over: bool) -> Input {
        Input { now, strip: true, peeking: false, over, blocked: false }
    }
    fn peek(now: u64, over: bool) -> Input {
        Input { now, strip: false, peeking: true, over, blocked: false }
    }

    fn run(mut p: Phase, inputs: &[Input]) -> (Phase, Vec<Action>) {
        let mut acts = vec![];
        for i in inputs {
            let (n, a) = step(p, *i);
            p = n;
            acts.push(a);
        }
        (p, acts)
    }

    fn polled(from: u64, to: u64, f: impl Fn(u64) -> Input) -> Vec<Input> {
        (from..=to).step_by(POLL_MS as usize).map(f).collect()
    }

    #[test]
    fn peek_opens_after_200ms_of_hover() {
        let (p, acts) = run(Phase::Away, &polled(0, 175, |t| strip(t, true)));
        assert!(acts.iter().all(|a| *a == Action::None));
        assert_eq!(p, Phase::Arming { since: 0 });
        assert_eq!(step(p, strip(200, true)), (Phase::Open, Action::Open));
    }

    #[test]
    fn peek_a_pass_under_200ms_opens_nothing() {
        let mut inputs = polled(0, 175, |t| strip(t, true));
        inputs.extend(polled(200, 600, |t| strip(t, false)));
        inputs.extend(polled(625, 800, |t| strip(t, true)));
        let (_, acts) = run(Phase::Away, &inputs);
        assert!(acts.iter().all(|a| *a == Action::None), "{acts:?}");
    }

    #[test]
    fn peek_folds_300ms_after_leaving_and_reentry_cancels() {
        let (p, _) = run(Phase::Open, &[peek(0, false), peek(250, false)]);
        assert_eq!(p, Phase::Closing { since: 0 });
        let (p, a) = step(p, peek(275, true));
        assert_eq!((p, a), (Phase::Open, Action::None));
        let (p, acts) = run(p, &polled(300, 575, |t| peek(t, false)));
        assert!(acts.iter().all(|a| *a == Action::None));
        assert_eq!(step(p, peek(600, false)), (Phase::Resting, Action::Close));
    }

    #[test]
    fn peek_stays_open_while_a_button_or_drag_holds_it() {
        let held = Input { blocked: true, ..peek(1000, false) };
        assert_eq!(step(Phase::Closing { since: 0 }, held), (Phase::Open, Action::None));
    }

    #[test]
    fn peek_never_arms_while_blocked_and_needs_a_fresh_entry_after() {
        let drag = |t| Input { blocked: true, ..strip(t, true) };
        let mut inputs = polled(0, 500, drag);
        inputs.extend(polled(525, 1000, |t| strip(t, true)));
        let (p, acts) = run(Phase::Away, &inputs);
        assert!(acts.iter().all(|a| *a == Action::None));
        assert_eq!(p, Phase::Resting);
        let (p, _) = step(p, strip(1025, false));
        let (p, _) = run(p, &polled(1050, 1225, |t| strip(t, true)));
        assert_eq!(step(p, strip(1250, true)), (Phase::Open, Action::Open));
    }

    #[test]
    fn peek_does_not_open_when_the_panel_collapses_under_the_cursor() {
        let panel = Input { strip: false, ..strip(0, true) };
        let (p, _) = step(Phase::Away, panel);
        let (p, acts) = run(p, &polled(25, 1000, |t| strip(t, true)));
        assert!(acts.iter().all(|a| *a == Action::None));
        assert_eq!(p, Phase::Resting);
    }

    #[test]
    fn peek_ended_from_outside_returns_to_rest() {
        // Pinned or folded by the hotkey: no longer peeking, no Close from here.
        let pinned = Input { strip: false, peeking: false, ..peek(0, false) };
        assert_eq!(step(Phase::Closing { since: 0 }, pinned), (Phase::Resting, Action::None));
    }
}
