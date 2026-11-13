use serde::Serialize;

pub const STUCK_AFTER_MS: u64 = 6 * 60 * 1000;
pub const HOOK_FRESH_MS: u64 = 10 * 1000;
pub const TARGET_MAX_CHARS: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Idle,
    Thinking,
    Working,
    Waiting,
    Success,
    Error,
    Stuck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LastTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub pid: u32,
    pub name: String,
    pub cwd: String,
    pub kind: String,
    pub entrypoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub state: State,
    pub state_since: u64,
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool: Option<LastTool>,
    pub subagents: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_pct: Option<f64>,
    pub alive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<u64>,
    /// Last activity; drives the stuck timer.
    #[serde(skip)]
    pub last_event_at: u64,
    #[serde(skip)]
    pub last_hook_at: Option<u64>,
    /// State to return to when a non-transition event ends `stuck`.
    #[serde(skip)]
    pub pre_stuck: Option<State>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    SessionStart,
    UserPromptSubmit,
    PreToolUse { tool: String, target: Option<String> },
    PostToolUse,
    /// `idle` marks the "waiting for a new prompt" reminder.
    Notification { message: String, idle: bool },
    Stop,
    StopFailure { summary: String },
    SubagentStop,
    /// `status` field of the session file: `busy` = true, `idle` = false.
    SessionFile { busy: bool },
    TranscriptToolUse { tool: String, target: Option<String> },
    TranscriptToolResult,
    TranscriptApiError { summary: String },
    Statusline { model: Option<String>, context_pct: Option<f64> },
    /// Process exited (PID dead) or `SessionEnd` hook.
    Ended,
    Tick,
}

impl Session {
    pub fn new(
        id: impl Into<String>,
        pid: u32,
        name: impl Into<String>,
        cwd: impl Into<String>,
        kind: impl Into<String>,
        entrypoint: impl Into<String>,
        now: u64,
    ) -> Self {
        Session {
            id: id.into(),
            pid,
            name: name.into(),
            cwd: cwd.into(),
            kind: kind.into(),
            entrypoint: entrypoint.into(),
            model: None,
            state: State::Idle,
            state_since: now,
            task: "Idle".into(),
            last_tool: None,
            subagents: 0,
            context_pct: None,
            alive: true,
            ended_at: None,
            last_event_at: now,
            last_hook_at: None,
            pre_stuck: None,
        }
    }

    fn hooks_fresh(&self, now: u64) -> bool {
        self.last_hook_at
            .is_some_and(|t| now.saturating_sub(t) <= HOOK_FRESH_MS)
    }

    fn enter(&mut self, state: State, task: String, now: u64) {
        if state != self.state {
            self.state_since = now;
        }
        if state != State::Stuck {
            self.pre_stuck = None;
        }
        self.state = state;
        self.task = task;
    }

    fn set_tool(&mut self, tool: String, target: Option<String>) {
        if is_subagent_tool(&tool) {
            self.subagents += 1;
        }
        let target = target.map(|t| truncate(&t, TARGET_MAX_CHARS));
        self.last_tool = Some(LastTool { name: tool, target });
    }

    fn working_task(&self) -> String {
        match &self.last_tool {
            Some(LastTool { name, target: Some(t) }) => format!("{name} {t}"),
            Some(LastTool { name, target: None }) => name.clone(),
            None => "Working".into(),
        }
    }
}

fn is_subagent_tool(tool: &str) -> bool {
    tool == "Task" || tool == "Agent"
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

fn stuck_task(since_event_ms: u64) -> String {
    format!("No response for {} min", since_event_ms / 60_000)
}

pub fn apply(mut s: Session, event: Event, now: u64) -> Session {
    if !s.alive {
        return s;
    }

    // Restores the pre-stuck state for events that define no transition of their own.
    let unstick = |s: &mut Session| {
        if s.state == State::Stuck {
            let back = s.pre_stuck.unwrap_or(State::Thinking);
            let task = match back {
                State::Working => s.working_task(),
                _ => "Thinking".into(),
            };
            s.enter(back, task, now);
        }
    };

    match event {
        Event::Tick => {
            let quiet = now.saturating_sub(s.last_event_at);
            match s.state {
                State::Thinking | State::Working if quiet >= STUCK_AFTER_MS => {
                    s.pre_stuck = Some(s.state);
                    s.state = State::Stuck;
                    s.state_since = now;
                    s.task = stuck_task(quiet);
                }
                State::Stuck => s.task = stuck_task(quiet),
                _ => {}
            }
            return s;
        }
        Event::Statusline { model, context_pct } => {
            if model.is_some() {
                s.model = model;
            }
            if context_pct.is_some() {
                s.context_pct = context_pct;
            }
            return s;
        }
        Event::Ended => {
            let st = if s.state == State::Success { State::Success } else { State::Idle };
            let task = if st == State::Success { "Done" } else { "Idle" };
            s.enter(st, task.into(), now);
            s.alive = false;
            s.ended_at = Some(now);
            return s;
        }
        Event::SessionFile { busy } => {
            if s.hooks_fresh(now) {
                return s;
            }
            let before = s.state;
            match (busy, s.state) {
                (true, State::Idle | State::Success | State::Error) => {
                    let task = s.working_task();
                    s.enter(State::Working, task, now);
                }
                (false, State::Thinking | State::Working | State::Stuck) => {
                    s.enter(State::Idle, "Idle".into(), now);
                }
                _ => {}
            }
            if s.state != before {
                s.last_event_at = now;
            }
            return s;
        }
        Event::TranscriptToolUse { .. } | Event::TranscriptToolResult if s.hooks_fresh(now) => {
            return s;
        }
        _ => {}
    }

    let is_hook = matches!(
        event,
        Event::SessionStart
            | Event::UserPromptSubmit
            | Event::PreToolUse { .. }
            | Event::PostToolUse
            | Event::Notification { .. }
            | Event::Stop
            | Event::StopFailure { .. }
            | Event::SubagentStop
    );
    if is_hook {
        s.last_hook_at = Some(now);
    }
    s.last_event_at = now;

    match event {
        Event::SessionStart => s.enter(State::Idle, "Idle".into(), now),
        Event::UserPromptSubmit => s.enter(State::Thinking, "Thinking".into(), now),
        Event::PreToolUse { tool, target } | Event::TranscriptToolUse { tool, target } => {
            s.set_tool(tool, target);
            if s.state != State::Success {
                let task = s.working_task();
                s.enter(State::Working, task, now);
            }
        }
        Event::PostToolUse | Event::TranscriptToolResult => {
            if s.state != State::Success {
                s.enter(State::Thinking, "Thinking".into(), now);
            }
        }
        Event::Notification { message, idle } => {
            if idle {
                unstick(&mut s);
            } else {
                let task = if message.trim().is_empty() {
                    "Waiting for input".to_string()
                } else {
                    message
                };
                s.enter(State::Waiting, task, now);
            }
        }
        Event::Stop => {
            s.subagents = 0;
            s.enter(State::Success, "Done".into(), now);
        }
        Event::StopFailure { summary } | Event::TranscriptApiError { summary } => {
            let task = if summary.trim().is_empty() { "Error".to_string() } else { summary };
            s.enter(State::Error, task, now);
        }
        Event::SubagentStop => {
            s.subagents = s.subagents.saturating_sub(1);
            unstick(&mut s);
        }
        Event::SessionFile { .. }
        | Event::Statusline { .. }
        | Event::Ended
        | Event::Tick => unreachable!("handled above"),
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: u64 = 1_000_000;
    const MIN: u64 = 60_000;

    fn fresh() -> Session {
        Session::new("s1", 42, "agentview", "F:/p", "interactive", "cli", T0)
    }

    fn pre(tool: &str, target: Option<&str>) -> Event {
        Event::PreToolUse { tool: tool.into(), target: target.map(Into::into) }
    }

    fn notif(msg: &str, idle: bool) -> Event {
        Event::Notification { message: msg.into(), idle }
    }

    fn run(events: Vec<(Event, u64)>) -> Session {
        events.into_iter().fold(fresh(), |s, (e, t)| apply(s, e, t))
    }

    // --- §4 table rows ---

    #[test]
    fn discovered_session_is_idle() {
        let s = fresh();
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.task, "Idle");
        assert_eq!(s.state_since, T0);
        assert!(s.alive);
    }

    #[test]
    fn session_start_is_idle() {
        let s = run(vec![(Event::Stop, T0 + 1), (Event::SessionStart, T0 + 2)]);
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.task, "Idle");
    }

    #[test]
    fn user_prompt_submit_is_thinking() {
        let s = run(vec![(Event::UserPromptSubmit, T0 + 1)]);
        assert_eq!(s.state, State::Thinking);
        assert_eq!(s.task, "Thinking");
        assert_eq!(s.state_since, T0 + 1);
    }

    #[test]
    fn post_tool_use_is_thinking() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0 + 1),
            (pre("Bash", Some("ls")), T0 + 2),
            (Event::PostToolUse, T0 + 3),
        ]);
        assert_eq!(s.state, State::Thinking);
        assert_eq!(s.task, "Thinking");
    }

    #[test]
    fn pre_tool_use_is_working_with_tool_and_target() {
        let s = run(vec![(Event::UserPromptSubmit, T0 + 1), (pre("Edit", Some("src/main.ts")), T0 + 2)]);
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Edit src/main.ts");
        assert_eq!(
            s.last_tool,
            Some(LastTool { name: "Edit".into(), target: Some("src/main.ts".into()) })
        );
    }

    #[test]
    fn pre_tool_use_without_target_shows_tool_only() {
        let s = run(vec![(pre("WebSearch", None), T0 + 1)]);
        assert_eq!(s.task, "WebSearch");
    }

    #[test]
    fn working_target_truncated_to_60_chars() {
        let long = "x".repeat(100);
        let s = run(vec![(pre("Bash", Some(&long)), T0 + 1)]);
        let target = s.last_tool.as_ref().unwrap().target.as_ref().unwrap();
        assert_eq!(target.chars().count(), 60);
        assert!(target.ends_with('…'));
        assert_eq!(s.task, format!("Bash {target}"));

        let exact = "y".repeat(60);
        let s = run(vec![(pre("Bash", Some(&exact)), T0 + 1)]);
        assert_eq!(s.task, format!("Bash {exact}"));
    }

    #[test]
    fn truncation_counts_chars_not_bytes() {
        let long = "é".repeat(70);
        let s = run(vec![(pre("Read", Some(&long)), T0 + 1)]);
        let target = s.last_tool.unwrap().target.unwrap();
        assert_eq!(target.chars().count(), 60);
    }

    #[test]
    fn notification_is_waiting_with_message() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0 + 1),
            (notif("Claude needs your permission to use Bash", false), T0 + 2),
        ]);
        assert_eq!(s.state, State::Waiting);
        assert_eq!(s.task, "Claude needs your permission to use Bash");
    }

    #[test]
    fn notification_with_empty_message_has_fallback_task() {
        let s = run(vec![(notif("  ", false), T0 + 1)]);
        assert_eq!(s.state, State::Waiting);
        assert_eq!(s.task, "Waiting for input");
    }

    #[test]
    fn stop_is_success_done() {
        let s = run(vec![(Event::UserPromptSubmit, T0 + 1), (Event::Stop, T0 + 2)]);
        assert_eq!(s.state, State::Success);
        assert_eq!(s.task, "Done");
    }

    #[test]
    fn stop_failure_is_error_with_summary() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0 + 1),
            (Event::StopFailure { summary: "API Error: 529 Overloaded".into() }, T0 + 2),
        ]);
        assert_eq!(s.state, State::Error);
        assert_eq!(s.task, "API Error: 529 Overloaded");
    }

    #[test]
    fn transcript_api_error_is_error_even_with_fresh_hooks() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0 + 1),
            (Event::TranscriptApiError { summary: "".into() }, T0 + 2),
        ]);
        assert_eq!(s.state, State::Error);
        assert_eq!(s.task, "Error");
    }

    #[test]
    fn error_leaves_on_next_prompt() {
        let s = run(vec![
            (Event::StopFailure { summary: "boom".into() }, T0 + 1),
            (Event::UserPromptSubmit, T0 + 2),
        ]);
        assert_eq!(s.state, State::Thinking);
    }

    // --- stuck ---

    #[test]
    fn thinking_goes_stuck_after_6_min_via_tick() {
        let s = run(vec![(Event::UserPromptSubmit, T0), (Event::Tick, T0 + 6 * MIN - 1)]);
        assert_eq!(s.state, State::Thinking);
        let s = apply(s, Event::Tick, T0 + 6 * MIN);
        assert_eq!(s.state, State::Stuck);
        assert_eq!(s.task, "No response for 6 min");
        assert_eq!(s.state_since, T0 + 6 * MIN);
    }

    #[test]
    fn working_goes_stuck_after_6_min() {
        let s = run(vec![(pre("Bash", Some("npm test")), T0), (Event::Tick, T0 + 6 * MIN)]);
        assert_eq!(s.state, State::Stuck);
    }

    #[test]
    fn stuck_task_counts_up_without_moving_state_since() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (Event::Tick, T0 + 6 * MIN),
            (Event::Tick, T0 + 9 * MIN + 30_000),
        ]);
        assert_eq!(s.state, State::Stuck);
        assert_eq!(s.task, "No response for 9 min");
        assert_eq!(s.state_since, T0 + 6 * MIN);
    }

    #[test]
    fn other_states_never_go_stuck() {
        for setup in [
            vec![],
            vec![(notif("perm", false), T0)],
            vec![(Event::Stop, T0)],
            vec![(Event::StopFailure { summary: "x".into() }, T0)],
        ] {
            let mut events = setup;
            events.push((Event::Tick, T0 + 60 * MIN));
            let before = run(events[..events.len() - 1].to_vec()).state;
            assert_eq!(run(events).state, before);
        }
    }

    #[test]
    fn any_event_resets_stuck_timer() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (Event::SubagentStop, T0 + 5 * MIN),
            (Event::Tick, T0 + 7 * MIN),
        ]);
        assert_eq!(s.state, State::Thinking);
        let s = apply(s, Event::Tick, T0 + 11 * MIN);
        assert_eq!(s.state, State::Stuck);
    }

    #[test]
    fn tick_and_statusline_do_not_reset_stuck_timer() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (Event::Tick, T0 + 3 * MIN),
            (Event::Statusline { model: None, context_pct: Some(10.0) }, T0 + 4 * MIN),
            (Event::Tick, T0 + 6 * MIN),
        ]);
        assert_eq!(s.state, State::Stuck);
    }

    #[test]
    fn stuck_leaves_on_next_event_with_its_transition() {
        let stuck = run(vec![(Event::UserPromptSubmit, T0), (Event::Tick, T0 + 7 * MIN)]);
        assert_eq!(apply(stuck.clone(), pre("Read", Some("a")), T0 + 8 * MIN).state, State::Working);
        assert_eq!(apply(stuck.clone(), Event::PostToolUse, T0 + 8 * MIN).state, State::Thinking);
        assert_eq!(apply(stuck.clone(), Event::Stop, T0 + 8 * MIN).state, State::Success);
        assert_eq!(apply(stuck, notif("perm", false), T0 + 8 * MIN).state, State::Waiting);
    }

    #[test]
    fn stuck_returns_to_previous_state_on_non_transition_event() {
        let stuck = run(vec![(pre("Bash", Some("make")), T0), (Event::Tick, T0 + 7 * MIN)]);
        let s = apply(stuck.clone(), Event::SubagentStop, T0 + 8 * MIN);
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Bash make");
        assert_eq!(s.state_since, T0 + 8 * MIN);
        let s = apply(stuck, notif("idle", true), T0 + 8 * MIN);
        assert_eq!(s.state, State::Working);
    }

    // --- waiting exits ---

    #[test]
    fn waiting_leaves_on_tool_prompt_or_stop() {
        let waiting = run(vec![(pre("Bash", Some("rm x")), T0 + 1), (notif("perm", false), T0 + 2)]);
        assert_eq!(apply(waiting.clone(), pre("Bash", Some("rm x")), T0 + 3).state, State::Working);
        assert_eq!(apply(waiting.clone(), Event::PostToolUse, T0 + 3).state, State::Thinking);
        assert_eq!(apply(waiting.clone(), Event::UserPromptSubmit, T0 + 3).state, State::Thinking);
        assert_eq!(apply(waiting, Event::Stop, T0 + 3).state, State::Success);
    }

    #[test]
    fn waiting_holds_on_other_events() {
        let waiting = run(vec![(notif("perm", false), T0 + 1)]);
        for e in [Event::SubagentStop, Event::Tick, notif("idle", true), Event::SessionFile { busy: true }] {
            assert_eq!(apply(waiting.clone(), e, T0 + 60 * MIN).state, State::Waiting);
        }
    }

    // --- success hold ---

    #[test]
    fn success_held_until_user_prompt_submit() {
        let done = run(vec![(Event::UserPromptSubmit, T0 + 1), (Event::Stop, T0 + 2)]);
        for e in [
            pre("Read", Some("a")),
            Event::PostToolUse,
            Event::SubagentStop,
            Event::Tick,
            Event::SessionFile { busy: false },
        ] {
            let s = apply(done.clone(), e, T0 + 60 * MIN);
            assert_eq!(s.state, State::Success);
            assert_eq!(s.task, "Done");
        }
        assert_eq!(apply(done, Event::UserPromptSubmit, T0 + 3).state, State::Thinking);
    }

    #[test]
    fn idle_reminder_notification_does_not_override_success() {
        let s = run(vec![
            (Event::Stop, T0 + 1),
            (notif("Claude is waiting for your input", true), T0 + 60_000),
        ]);
        assert_eq!(s.state, State::Success);
        assert_eq!(s.task, "Done");
        assert_eq!(s.state_since, T0 + 1);
    }

    #[test]
    fn idle_reminder_never_changes_state() {
        let s = run(vec![(notif("waiting", true), T0 + 1)]);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn permission_notification_overrides_success() {
        let s = run(vec![(Event::Stop, T0 + 1), (notif("perm", false), T0 + 2)]);
        assert_eq!(s.state, State::Waiting);
    }

    // --- subagents ---

    #[test]
    fn subagent_count_increments_decrements_clamps_resets() {
        let s = run(vec![
            (pre("Task", Some("explore")), T0 + 1),
            (pre("Agent", Some("review")), T0 + 2),
            (pre("Bash", Some("ls")), T0 + 3),
        ]);
        assert_eq!(s.subagents, 2);
        let s = apply(s, Event::SubagentStop, T0 + 4);
        assert_eq!(s.subagents, 1);
        let s = apply(apply(s, Event::SubagentStop, T0 + 5), Event::SubagentStop, T0 + 6);
        assert_eq!(s.subagents, 0);
        let s = apply(apply(s, pre("Task", None), T0 + 7), pre("Task", None), T0 + 8);
        assert_eq!(s.subagents, 2);
        let s = apply(s, Event::Stop, T0 + 9);
        assert_eq!(s.subagents, 0);
    }

    // --- hook-less fallback ---

    #[test]
    fn session_file_busy_idle_maps_without_hooks() {
        let s = run(vec![(Event::SessionFile { busy: true }, T0 + 1)]);
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Working");
        let s = apply(s, Event::SessionFile { busy: false }, T0 + 2);
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.task, "Idle");
    }

    #[test]
    fn session_file_ignored_within_10s_of_a_hook() {
        let s = run(vec![(Event::Stop, T0), (Event::SessionFile { busy: true }, T0 + HOOK_FRESH_MS)]);
        assert_eq!(s.state, State::Success);
        let s = run(vec![(Event::UserPromptSubmit, T0), (Event::SessionFile { busy: false }, T0 + 5_000)]);
        assert_eq!(s.state, State::Thinking);
    }

    #[test]
    fn session_file_overrides_stale_hook_state() {
        let s = run(vec![
            (pre("Bash", Some("ls")), T0),
            (Event::SessionFile { busy: false }, T0 + HOOK_FRESH_MS + 1),
        ]);
        assert_eq!(s.state, State::Idle);
    }

    #[test]
    fn session_file_never_overrides_waiting_success_or_busy_refinement() {
        let w = run(vec![(notif("perm", false), T0), (Event::SessionFile { busy: false }, T0 + MIN)]);
        assert_eq!(w.state, State::Waiting);
        let d = run(vec![(Event::Stop, T0), (Event::SessionFile { busy: false }, T0 + MIN)]);
        assert_eq!(d.state, State::Success);
        let t = run(vec![(Event::UserPromptSubmit, T0), (Event::SessionFile { busy: true }, T0 + MIN)]);
        assert_eq!(t.state, State::Thinking);
    }

    #[test]
    fn redundant_session_file_status_does_not_reset_stuck_timer() {
        let s = run(vec![
            (Event::SessionFile { busy: true }, T0),
            (Event::SessionFile { busy: true }, T0 + 5 * MIN),
            (Event::Tick, T0 + 6 * MIN),
        ]);
        assert_eq!(s.state, State::Stuck);
    }

    #[test]
    fn transcript_refines_hookless_sessions() {
        let s = run(vec![
            (Event::SessionFile { busy: true }, T0),
            (Event::TranscriptToolUse { tool: "Grep".into(), target: Some("TODO".into()) }, T0 + 1),
        ]);
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Grep TODO");
        let s = apply(s, Event::TranscriptToolResult, T0 + 2);
        assert_eq!(s.state, State::Thinking);
    }

    #[test]
    fn transcript_tool_events_ignored_while_hooks_fresh() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (Event::TranscriptToolUse { tool: "Task".into(), target: None }, T0 + 1),
        ]);
        assert_eq!(s.state, State::Thinking);
        assert_eq!(s.subagents, 0);
        assert_eq!(s.last_tool, None);
    }

    // --- metadata, lifecycle, timestamps ---

    #[test]
    fn statusline_fills_model_and_context_without_state_change() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (Event::Statusline { model: Some("Opus".into()), context_pct: Some(42.5) }, T0 + 1),
            (Event::Statusline { model: None, context_pct: None }, T0 + 2),
        ]);
        assert_eq!(s.state, State::Thinking);
        assert_eq!(s.model.as_deref(), Some("Opus"));
        assert_eq!(s.context_pct, Some(42.5));
    }

    #[test]
    fn ended_keeps_success_else_idle_and_freezes() {
        let done = run(vec![(Event::Stop, T0), (Event::Ended, T0 + 1)]);
        assert_eq!(done.state, State::Success);
        assert!(!done.alive);
        assert_eq!(done.ended_at, Some(T0 + 1));

        let busy = run(vec![(pre("Bash", None), T0), (Event::Ended, T0 + 1)]);
        assert_eq!(busy.state, State::Idle);
        assert_eq!(busy.task, "Idle");

        let after = apply(busy.clone(), Event::UserPromptSubmit, T0 + 2);
        assert_eq!(after, busy);
        assert_eq!(apply(busy.clone(), Event::Tick, T0 + 60 * MIN), busy);
    }

    #[test]
    fn state_since_moves_only_on_state_change() {
        let s = run(vec![(pre("Read", Some("a")), T0 + 1), (pre("Edit", Some("b")), T0 + 2)]);
        assert_eq!(s.state_since, T0 + 1);
        assert_eq!(s.task, "Edit b");
    }

    #[test]
    fn serialises_to_snapshot_shape() {
        let s = run(vec![
            (pre("Edit", Some("src/main.ts")), T0 + 1),
            (Event::Statusline { model: None, context_pct: Some(12.0) }, T0 + 2),
        ]);
        let v = serde_json::to_value(&s).unwrap();
        let obj = v.as_object().unwrap();
        assert_eq!(v["state"], "working");
        assert_eq!(v["stateSince"], T0 + 1);
        assert_eq!(v["lastTool"]["name"], "Edit");
        assert_eq!(v["lastTool"]["target"], "src/main.ts");
        assert_eq!(v["contextPct"], 12.0);
        assert_eq!(v["subagents"], 0);
        assert_eq!(v["alive"], true);
        for key in ["id", "pid", "name", "cwd", "kind", "entrypoint", "task"] {
            assert!(obj.contains_key(key), "missing {key}");
        }
        for absent in ["model", "endedAt", "lastEventAt", "lastHookAt", "preStuck", "last_event_at"] {
            assert!(!obj.contains_key(absent), "unexpected {absent}");
        }
        let states: Vec<_> = [
            State::Idle,
            State::Thinking,
            State::Working,
            State::Waiting,
            State::Success,
            State::Error,
            State::Stuck,
        ]
        .iter()
        .map(|st| serde_json::to_value(st).unwrap())
        .collect();
        assert_eq!(
            states,
            ["idle", "thinking", "working", "waiting", "success", "error", "stuck"]
        );
    }
}
