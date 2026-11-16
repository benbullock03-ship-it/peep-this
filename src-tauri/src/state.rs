use serde::{Deserialize, Serialize};

use crate::approve::PendingApproval;
use crate::mood::Mood;
use crate::tip::{self, Tip};

pub const STUCK_AFTER_MS: u64 = 6 * 60 * 1000;
pub const HOOK_FRESH_MS: u64 = 10 * 1000;
pub const TARGET_MAX_CHARS: usize = 60;
/// Finished agents kept per session; the oldest are dropped first.
pub const FINISHED_AGENTS_KEPT: usize = 20;
const PENDING_AGENTS_MAX: usize = 8;
/// A running shell whose output file has not changed for this long is `quiet`.
pub const SHELL_QUIET_MS: u64 = 2 * 60 * 1000;
pub const FINISHED_SHELLS_KEPT: usize = 20;
/// Peep skills that link a session (spec §13.1); any other `peep:` name is ignored.
pub const PEEP_SKILLS: [&str; 11] =
    ["ticket", "ducks", "qt", "packup", "shoot", "coderot", "walk", "sneak", "bigpeep", "tweet", "goose"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// The tool input's own `description` (Bash/PowerShell), for the speech bubble.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketLink {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub pid: u32,
    pub name: String,
    /// User-set name, else the ai-title; shown over `name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Branch of `cwd`'s repo (short sha when detached); the store refreshes it on state change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
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
    pub agents: Vec<Agent>,
    pub shells: Vec<Shell>,
    /// Held permission requests; owned by the approval broker, never touched by `apply`.
    pub pending: Vec<PendingApproval>,
    /// Peep skill running this turn, without the `peep:` namespace; cleared on `Stop`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    /// Kept across turns until the next `/peep:` prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<TicketLink>,
    /// Value of the last `next:` line since the latest Peep skill started (formats.md §8); kept across turns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// A Peep skill has run since the widget started tracking this session; `next:` lines count only then.
    #[serde(skip)]
    pub peep_ran: bool,
    /// Set by the mood engine in the store, never by `apply`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mood: Option<Mood>,
    /// Absolute path of the clashed file while `mood` is startled; set by the mood engine with `mood`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_pct: Option<f64>,
    /// Signal tip on show; the store sets it from `signal`, a prompt or a dismissal clears it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tip: Option<Tip>,
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
    /// Statusline display names win over transcript model ids once seen.
    #[serde(skip)]
    pub model_from_statusline: bool,
    /// Descriptions of `Agent` calls not yet bound to an `agent_id`, oldest first.
    #[serde(skip)]
    pub pending_agents: Vec<String>,
    /// Set while `waiting` came from a subagent's prompt: (agent id, state and task to return to).
    #[serde(skip)]
    pub waiting_on_agent: Option<(String, State, String)>,
    #[serde(skip)]
    pub born_at: u64,
    /// Title sources, highest precedence first: session file (user-named), transcript `custom-title`, `ai-title`.
    #[serde(skip)]
    pub file_title: Option<String>,
    #[serde(skip)]
    pub custom_title: Option<String>,
    #[serde(skip)]
    pub ai_title: Option<String>,
    /// Last failed run of the repo setup's test command; cleared when one passes.
    #[serde(skip)]
    pub worried_at: Option<u64>,
    /// (absolute path, last edit) within the clash window, for the clash mood.
    #[serde(skip)]
    pub edits: Vec<(String, u64)>,
    /// Raised by `apply`, taken by the store on every mutation.
    #[serde(skip)]
    pub signal: Option<Tip>,
    #[serde(skip)]
    pub context_tokens: Option<u64>,
    #[serde(skip)]
    pub cache_ttl_ms: Option<u64>,
    /// Failures per tool name since the last prompt.
    #[serde(skip)]
    pub fails: Vec<(String, u32)>,
    #[serde(skip)]
    pub crowded_raised: bool,
    /// Once per idle stretch; a prompt resets it.
    #[serde(skip)]
    pub cold_raised: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub agent_type: String,
    pub description: String,
    pub state: State,
    pub state_since: u64,
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool: Option<LastTool>,
    pub started_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<u64>,
    pub tool_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip)]
    pub last_event_at: u64,
    #[serde(skip)]
    pub pre_stuck: Option<State>,
    /// `SubagentHandback` report; wins over `last_assistant_message` as the result.
    #[serde(skip)]
    pub handback: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellState {
    Running,
    Quiet,
    Serving,
    Exited,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shell {
    /// Claude Code's background task id.
    pub id: String,
    pub command: String,
    pub description: String,
    pub state: ShellState,
    pub state_since: u64,
    pub started_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_line: Option<String>,
    /// OS PID of the shell's wrapper process once the probe has matched it.
    #[serde(skip)]
    pub pid: Option<u32>,
    /// Set once a probe has run after the shell ended, so its last output line is read once.
    #[serde(skip)]
    pub tail_read: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellExit {
    Code(i32),
    /// Failed with no code reported.
    Failed,
    Killed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShellEvent {
    /// `PostToolUse` of `Bash`/`PowerShell` whose response carries `backgroundTaskId`.
    Started { id: String, command: String, description: String },
    /// `<task-notification>` prompt or `TaskStop`.
    Ended { id: String, exit: ShellExit },
    /// Shell entries of `Stop.background_tasks` as (id, command, description); unknown ids are adopted.
    Listed { shells: Vec<(String, String, String)> },
}

/// What the OS probe saw for one shell (spec §11.7); every field is best effort.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellObservation {
    pub pid: Option<u32>,
    pub port: Option<u16>,
    /// Last write to the shell's output file, epoch ms.
    pub last_output_at: Option<u64>,
    pub last_line: Option<String>,
    /// From the output file's `[exited with code N]` / `[killed]` trailer.
    pub exit: Option<ShellExit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalOutcome {
    Allowed,
    Denied,
    /// The terminal took the request and no `PostToolUse` followed: it said no.
    Abandoned,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    SessionStart,
    UserPromptSubmit,
    PreToolUse { tool: String, target: Option<String>, description: Option<String> },
    PostToolUse,
    /// `idle` marks the "waiting for a new prompt" reminder.
    Notification { message: String, idle: bool },
    Stop,
    StopFailure { summary: String },
    SubagentStart { agent_id: String, agent_type: String },
    /// `agent_id` is unknown for Claude Code's internal agents; `result` is the first line of the final message.
    SubagentStop { agent_id: Option<String>, result: Option<String> },
    /// A tool or notification event fired inside a subagent (payload had `agent_id`).
    Subagent { agent_id: String, agent_type: String, event: Box<Event> },
    /// `status` field of the session file: `busy` = true, `idle` = false.
    SessionFile { busy: bool },
    TranscriptToolUse { tool: String, target: Option<String> },
    TranscriptToolResult,
    TranscriptApiError { summary: String },
    TranscriptModel { model: String },
    /// `user`: a `/rename` title; otherwise the ai-title.
    TranscriptTitle { title: String, user: bool },
    /// The value of a `next:` line ending an assistant text block.
    TranscriptNext { line: String },
    Statusline { model: Option<String>, context_pct: Option<f64>, tokens: Option<u64>, cache_ttl_ms: Option<u64> },
    /// A main-session `PostToolUseFailure` that was not an interrupt.
    ToolFailed { tool: String, event: Box<Event> },
    /// An auto-compact boundary in the transcript.
    TranscriptCompacted,
    /// A hook event that also carries a background shell update; `event` is applied as usual afterwards.
    Shell { update: ShellEvent, event: Box<Event> },
    /// A `/peep:<skill>` prompt or `Skill` tool call; `event` is applied as usual first.
    Skill { skill: String, ticket: Option<TicketLink>, event: Box<Event> },
    /// A finished Bash/PowerShell call that ran the repo setup's test command; `event` is applied as usual afterwards.
    TestRun { passed: bool, event: Box<Event> },
    /// A successful Edit/Write/MultiEdit/NotebookEdit with the file's absolute path; `event` is applied as usual afterwards.
    Edited { path: String, event: Box<Event> },
    /// OS probe result for one shell; never resets the stuck timer.
    ShellProbe { id: String, obs: ShellObservation },
    /// The widget answered a held permission request, or the terminal took it (spec §11.1).
    /// Needed because no `PostToolUse` follows a deny.
    Approval { agent_id: Option<String>, outcome: ApprovalOutcome },
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
            title: None,
            branch: None,
            cwd: cwd.into(),
            kind: kind.into(),
            entrypoint: entrypoint.into(),
            model: None,
            state: State::Idle,
            state_since: now,
            task: "Idle".into(),
            last_tool: None,
            subagents: 0,
            agents: Vec::new(),
            shells: Vec::new(),
            pending: Vec::new(),
            skill: None,
            ticket: None,
            next: None,
            peep_ran: false,
            mood: None,
            clash: None,
            context_pct: None,
            tip: None,
            alive: true,
            ended_at: None,
            last_event_at: now,
            last_hook_at: None,
            pre_stuck: None,
            model_from_statusline: false,
            pending_agents: Vec::new(),
            waiting_on_agent: None,
            born_at: now,
            file_title: None,
            custom_title: None,
            ai_title: None,
            worried_at: None,
            edits: Vec::new(),
            signal: None,
            context_tokens: None,
            cache_ttl_ms: None,
            fails: Vec::new(),
            crowded_raised: false,
            cold_raised: false,
        }
    }

    /// Idle past the cache lifetime after at least one request (a statusline reported usage).
    pub fn cold_due(&self, now: u64) -> bool {
        self.alive
            && !self.cold_raised
            && matches!(self.state, State::Idle | State::Success)
            && (self.context_tokens.is_some() || self.context_pct.is_some())
            && now.saturating_sub(self.last_event_at) >= self.cache_ttl_ms.unwrap_or(tip::CACHE_TTL_DEFAULT_MS)
    }

    pub fn refresh_title(&mut self) {
        self.title = self.file_title.clone().or_else(|| self.custom_title.clone()).or_else(|| self.ai_title.clone());
    }

    /// The agent with this id, created if unknown; a new one takes the oldest pending `Agent` description.
    fn agent_mut(&mut self, id: &str, agent_type: &str, now: u64) -> &mut Agent {
        let i = match self.agents.iter().position(|a| a.id == id) {
            Some(i) => i,
            None => {
                let description = if self.pending_agents.is_empty() {
                    String::new()
                } else {
                    self.pending_agents.remove(0)
                };
                self.agents.push(Agent::new(id, agent_type, description, now));
                self.agents.len() - 1
            }
        };
        &mut self.agents[i]
    }

    fn running_agent_mut(&mut self, id: &str) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == id && a.ended_at.is_none())
    }

    fn drop_old_finished_agents(&mut self) {
        let finished = self.agents.iter().filter(|a| a.ended_at.is_some()).count();
        let mut excess = finished.saturating_sub(FINISHED_AGENTS_KEPT);
        self.agents.retain(|a| {
            if excess > 0 && a.ended_at.is_some() {
                excess -= 1;
                return false;
            }
            true
        });
    }

    fn push_pending_agent(&mut self, description: Option<String>) {
        if self.pending_agents.len() >= PENDING_AGENTS_MAX {
            self.pending_agents.remove(0);
        }
        self.pending_agents.push(description.unwrap_or_default());
    }

    /// Leaves a `waiting` that this agent's prompt caused once the agent moves on.
    fn release_agent_wait(&mut self, agent_id: &str, now: u64) -> bool {
        if self.state != State::Waiting || self.waiting_on_agent.as_ref().is_none_or(|w| w.0 != agent_id) {
            return false;
        }
        let (_, back, task) = self.waiting_on_agent.take().expect("checked above");
        let (back, task) = match back {
            State::Stuck | State::Waiting => (State::Working, self.working_task()),
            _ => (back, task),
        };
        self.enter(back, task, now);
        true
    }

    fn notify(&mut self, message: String, now: u64) {
        if self.state == State::Waiting && !self.task.trim().is_empty() {
            // The permission_prompt Notification ("Claude needs your permission") trails PermissionRequest; keep its tool text.
            return;
        }
        self.waiting_on_agent = None;
        self.enter(State::Waiting, waiting_task(message), now);
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
        if state != State::Waiting {
            self.waiting_on_agent = None;
        }
        self.state = state;
        self.task = task;
    }

    fn set_tool(&mut self, tool: String, target: Option<String>, description: Option<String>, count_subagent: bool) {
        if count_subagent && is_subagent_tool(&tool) {
            self.subagents += 1;
        }
        let target = target.map(|t| truncate(&t, TARGET_MAX_CHARS));
        self.last_tool = Some(LastTool { name: tool, target, description });
    }

    fn working_task(&self) -> String {
        match &self.last_tool {
            Some(LastTool { name, target: Some(t), .. }) => format!("{name} {t}"),
            Some(LastTool { name, target: None, .. }) => name.clone(),
            None => "Working".into(),
        }
    }

    fn apply_shell(&mut self, update: ShellEvent, now: u64) {
        match update {
            ShellEvent::Started { id, command, description } => self.add_shell(id, command, description, now),
            ShellEvent::Listed { shells } => {
                for (id, command, description) in shells {
                    self.add_shell(id, command, description, now);
                }
            }
            ShellEvent::Ended { id, exit } => {
                if let Some(sh) = self.shells.iter_mut().find(|sh| sh.id == id && sh.ended_at.is_none()) {
                    sh.finish(exit, now);
                    self.drop_old_finished_shells();
                }
            }
        }
    }

    fn add_shell(&mut self, id: String, command: String, description: String, now: u64) {
        if !self.shells.iter().any(|sh| sh.id == id) {
            self.shells.push(Shell::new(id, command, description, now));
        }
    }

    fn probe_shell(&mut self, id: &str, obs: ShellObservation, now: u64) {
        let Some(sh) = self.shells.iter_mut().find(|sh| sh.id == id) else { return };
        if sh.ended_at.is_some() {
            if sh.last_line.is_none() {
                sh.last_line = obs.last_line;
            }
            sh.tail_read = true;
            return;
        }
        if obs.last_line.is_some() {
            sh.last_line = obs.last_line;
        }
        if obs.pid.is_some() {
            sh.pid = obs.pid;
        }
        if let Some(exit) = obs.exit {
            sh.finish(exit, now);
            sh.tail_read = true;
            self.drop_old_finished_shells();
            return;
        }
        sh.port = obs.port;
        let quiet = obs.last_output_at.is_some_and(|t| now.saturating_sub(t) >= SHELL_QUIET_MS);
        let state = match (sh.port, quiet) {
            (Some(_), _) => ShellState::Serving,
            (None, true) => ShellState::Quiet,
            (None, false) => ShellState::Running,
        };
        sh.enter(state, now);
    }

    fn drop_old_finished_shells(&mut self) {
        let finished = self.shells.iter().filter(|sh| sh.ended_at.is_some()).count();
        let mut excess = finished.saturating_sub(FINISHED_SHELLS_KEPT);
        self.shells.retain(|sh| {
            if excess > 0 && sh.ended_at.is_some() {
                excess -= 1;
                return false;
            }
            true
        });
    }
}

impl Shell {
    pub fn new(id: impl Into<String>, command: impl Into<String>, description: impl Into<String>, now: u64) -> Self {
        Shell {
            id: id.into(),
            command: command.into(),
            description: description.into(),
            state: ShellState::Running,
            state_since: now,
            started_at: now,
            ended_at: None,
            port: None,
            exit_code: None,
            last_line: None,
            pid: None,
            tail_read: false,
        }
    }

    fn enter(&mut self, state: ShellState, now: u64) {
        if state != self.state {
            self.state_since = now;
        }
        self.state = state;
    }

    fn finish(&mut self, exit: ShellExit, now: u64) {
        let (state, code) = match exit {
            ShellExit::Code(0) => (ShellState::Exited, Some(0)),
            ShellExit::Code(c) => (ShellState::Failed, Some(c)),
            ShellExit::Failed => (ShellState::Failed, None),
            ShellExit::Killed | ShellExit::Unknown => (ShellState::Exited, None),
        };
        self.enter(state, now);
        self.exit_code = code;
        self.port = None;
        self.ended_at = Some(now);
    }

    pub fn is_running(&self) -> bool {
        self.ended_at.is_none()
    }
}

impl Agent {
    pub fn new(id: impl Into<String>, agent_type: impl Into<String>, description: String, now: u64) -> Self {
        Agent {
            id: id.into(),
            agent_type: agent_type.into(),
            description,
            state: State::Thinking,
            state_since: now,
            task: "Thinking".into(),
            last_tool: None,
            started_at: now,
            ended_at: None,
            tool_count: 0,
            result: None,
            last_event_at: now,
            pre_stuck: None,
            handback: None,
        }
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

    fn working_task(&self) -> String {
        match &self.last_tool {
            Some(LastTool { name, target: Some(t), .. }) => format!("{name} {t}"),
            Some(LastTool { name, target: None, .. }) => name.clone(),
            None => "Working".into(),
        }
    }

    fn unstick(&mut self, now: u64) {
        if self.state == State::Stuck {
            let back = self.pre_stuck.unwrap_or(State::Thinking);
            let task = if back == State::Working { self.working_task() } else { "Thinking".into() };
            self.enter(back, task, now);
        }
    }

    /// The agent's own transitions: the parent's rules for tool and notification events.
    fn apply(&mut self, event: &Event, now: u64) {
        self.last_event_at = now;
        match event {
            Event::PreToolUse { tool, target, description } => {
                self.tool_count += 1;
                if tool == "SubagentHandback" {
                    self.handback = target.clone();
                }
                let target = target.as_ref().map(|t| truncate(t, TARGET_MAX_CHARS));
                self.last_tool = Some(LastTool { name: tool.clone(), target, description: description.clone() });
                let task = self.working_task();
                self.enter(State::Working, task, now);
            }
            Event::PostToolUse => self.enter(State::Thinking, "Thinking".into(), now),
            Event::Notification { idle: true, .. } => self.unstick(now),
            Event::Notification { message, idle: false } => {
                if !(self.state == State::Waiting && !self.task.trim().is_empty()) {
                    self.enter(State::Waiting, waiting_task(message.clone()), now);
                }
            }
            _ => self.unstick(now),
        }
    }

    fn finish(&mut self, result: Option<String>, now: u64) {
        self.last_event_at = now;
        self.result = self.handback.take().or(result).filter(|r| !r.trim().is_empty());
        self.enter(State::Success, "Done".into(), now);
        self.ended_at = Some(now);
    }

    fn tick(&mut self, now: u64) {
        if self.ended_at.is_some() {
            return;
        }
        let quiet = now.saturating_sub(self.last_event_at);
        match self.state {
            State::Thinking | State::Working if quiet >= STUCK_AFTER_MS => {
                self.pre_stuck = Some(self.state);
                self.state = State::Stuck;
                self.state_since = now;
                self.task = stuck_task(quiet);
            }
            State::Stuck => self.task = stuck_task(quiet),
            _ => {}
        }
    }

    pub fn is_busy(&self) -> bool {
        self.ended_at.is_none() && matches!(self.state, State::Thinking | State::Working | State::Stuck)
    }
}

fn waiting_task(message: String) -> String {
    if message.trim().is_empty() {
        "Waiting for input".to_string()
    } else {
        message
    }
}

fn is_subagent_tool(tool: &str) -> bool {
    tool == "Task" || tool == "Agent"
}

/// Cuts from the middle so a path keeps its filename.
fn truncate(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let keep = max - 1;
    let head = keep.div_ceil(2);
    let mut out: String = s.chars().take(head).collect();
    out.push('…');
    out.extend(s.chars().skip(n - (keep - head)));
    out
}

fn stuck_task(since_event_ms: u64) -> String {
    format!("No response for {} min", since_event_ms / 60_000)
}

pub fn apply(mut s: Session, event: Event, now: u64) -> Session {
    if !s.alive {
        return s;
    }
    if let Event::Shell { update, event } = event {
        s.apply_shell(update, now);
        return apply(s, *event, now);
    }
    if let Event::TestRun { passed, event } = event {
        s.worried_at = (!passed).then_some(now);
        return apply(s, *event, now);
    }
    if let Event::ToolFailed { tool, event } = event {
        let n = match s.fails.iter_mut().find(|(t, _)| *t == tool) {
            Some((_, n)) => {
                *n += 1;
                *n
            }
            None => {
                s.fails.push((tool, 1));
                1
            }
        };
        if n == tip::MISSES {
            s.signal = Some(Tip::Misses);
        }
        return apply(s, *event, now);
    }
    if let Event::Edited { path, event } = event {
        s.edits.retain(|(p, at)| !crate::mood::same_path(p, &path) && now.saturating_sub(*at) <= crate::mood::CLASH_WINDOW_MS);
        s.edits.push((path, now));
        return apply(s, *event, now);
    }
    if let Event::Skill { skill, ticket, event } = event {
        // A prompt replaces the ticket link; a Skill tool call mid-turn only adds one.
        let from_prompt = *event == Event::UserPromptSubmit;
        let mut s = apply(s, *event, now);
        if s.alive && PEEP_SKILLS.contains(&skill.as_str()) {
            if from_prompt || ticket.is_some() {
                s.ticket = ticket;
            }
            if s.skill.as_deref() != Some(skill.as_str()) {
                s.next = None;
            }
            s.peep_ran = true;
            s.skill = Some(skill);
        }
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
            s.agents.iter_mut().for_each(|a| a.tick(now));
            let quiet = now.saturating_sub(s.last_event_at);
            match s.state {
                State::Thinking | State::Working if quiet >= STUCK_AFTER_MS => {
                    s.pre_stuck = Some(s.state);
                    s.state = State::Stuck;
                    s.state_since = now;
                    s.task = stuck_task(quiet);
                }
                State::Stuck => s.task = stuck_task(quiet),
                State::Idle | State::Success if s.cold_due(now) => {
                    s.cold_raised = true;
                    s.signal = Some(Tip::Cold);
                }
                _ => {}
            }
            return s;
        }
        Event::Statusline { model, context_pct, tokens, cache_ttl_ms } => {
            if model.is_some() {
                s.model = model;
                s.model_from_statusline = true;
            }
            if context_pct.is_some() {
                s.context_pct = context_pct;
            }
            if tokens.is_some() {
                s.context_tokens = tokens;
            }
            if cache_ttl_ms.is_some() {
                s.cache_ttl_ms = cache_ttl_ms;
            }
            if !s.crowded_raised && tip::crowded(s.context_tokens, s.context_pct) {
                s.crowded_raised = true;
                s.signal = Some(Tip::Crowded);
            }
            return s;
        }
        Event::TranscriptCompacted => {
            s.signal = Some(Tip::Compacted);
            return s;
        }
        Event::TranscriptModel { model } => {
            if !s.model_from_statusline {
                s.model = Some(model);
            }
            return s;
        }
        Event::TranscriptTitle { title, user } => {
            if user {
                s.custom_title = Some(title);
            } else {
                s.ai_title = Some(title);
            }
            s.refresh_title();
            return s;
        }
        Event::TranscriptNext { line } => {
            if s.peep_ran {
                s.next = Some(line);
            }
            return s;
        }
        Event::ShellProbe { id, obs } => {
            s.probe_shell(&id, obs, now);
            return s;
        }
        Event::Ended => {
            let st = if s.state == State::Success { State::Success } else { State::Idle };
            let task = if st == State::Success { "Done" } else { "Idle" };
            s.enter(st, task.into(), now);
            s.alive = false;
            s.ended_at = Some(now);
            s.skill = None;
            s.ticket = None;
            for a in s.agents.iter_mut().filter(|a| a.ended_at.is_none()) {
                a.enter(State::Idle, "Idle".into(), now);
                a.ended_at = Some(now);
            }
            // Claude Code stops its background tasks on exit.
            for sh in s.shells.iter_mut().filter(|sh| sh.ended_at.is_none()) {
                sh.finish(ShellExit::Unknown, now);
            }
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
            | Event::SubagentStart { .. }
            | Event::SubagentStop { .. }
            | Event::Subagent { .. }
    );
    if is_hook {
        s.last_hook_at = Some(now);
    }
    s.last_event_at = now;

    match event {
        Event::SessionStart => s.enter(State::Idle, "Idle".into(), now),
        Event::UserPromptSubmit => {
            s.skill = None;
            s.tip = None;
            s.fails.clear();
            s.cold_raised = false;
            s.enter(State::Thinking, "Thinking".into(), now);
        }
        Event::PreToolUse { .. } | Event::TranscriptToolUse { .. } => {
            let (tool, target, description) = match event {
                Event::PreToolUse { tool, target, description } => (tool, target, description),
                Event::TranscriptToolUse { tool, target } => (tool, target, None),
                _ => unreachable!(),
            };
            if is_hook && is_subagent_tool(&tool) {
                s.push_pending_agent(target.clone());
            }
            // Only hooks count subagents: SubagentStop is hook-only, and a transcript line racing its PreToolUse would count twice.
            s.set_tool(tool, target, description, is_hook);
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
                s.notify(message, now);
            }
        }
        Event::Stop => {
            // Background agents outlive the turn, so `agents` is left alone.
            s.subagents = 0;
            s.pending_agents.clear();
            s.skill = None;
            s.enter(State::Success, "Done".into(), now);
        }
        Event::StopFailure { summary } | Event::TranscriptApiError { summary } => {
            let task = if summary.trim().is_empty() { "Error".to_string() } else { summary };
            s.enter(State::Error, task, now);
        }
        Event::SubagentStart { agent_id, agent_type } => {
            s.signal = Some(Tip::Helper);
            let a = s.agent_mut(&agent_id, &agent_type, now);
            if a.ended_at.is_some() {
                // Resumed subagent or agent-team teammate taking a new message.
                *a = Agent { description: a.description.clone(), ..Agent::new(&agent_id, agent_type, String::new(), now) };
            }
            unstick(&mut s);
        }
        Event::Subagent { agent_id, agent_type, event } => {
            let parent_was_waiting = s.state == State::Waiting;
            let prior = (s.state, s.task.clone());
            let a = s.agent_mut(&agent_id, &agent_type, now);
            if a.ended_at.is_none() {
                a.apply(&event, now);
            }
            match *event {
                Event::Notification { message, idle: false } => {
                    // The parent row carries the alert; its own task stays put underneath.
                    s.notify(message, now);
                    if !parent_was_waiting {
                        s.waiting_on_agent = Some((agent_id, prior.0, prior.1));
                    }
                }
                Event::PreToolUse { tool, target, .. } if is_subagent_tool(&tool) => {
                    s.push_pending_agent(target);
                    if !s.release_agent_wait(&agent_id, now) {
                        unstick(&mut s);
                    }
                }
                _ => {
                    if !s.release_agent_wait(&agent_id, now) {
                        unstick(&mut s);
                    }
                }
            }
        }
        Event::SubagentStop { agent_id, result } => {
            s.subagents = s.subagents.saturating_sub(1);
            if let Some(id) = agent_id {
                if let Some(a) = s.running_agent_mut(&id) {
                    a.finish(result, now);
                    s.drop_old_finished_agents();
                }
                s.release_agent_wait(&id, now);
            }
            unstick(&mut s);
        }
        Event::Approval { agent_id, outcome } => match agent_id {
            Some(id) => {
                if let Some(a) = s.running_agent_mut(&id).filter(|a| a.state == State::Waiting) {
                    match outcome {
                        ApprovalOutcome::Allowed => {
                            let task = a.working_task();
                            a.enter(State::Working, task, now);
                        }
                        ApprovalOutcome::Denied | ApprovalOutcome::Abandoned => {
                            a.enter(State::Thinking, "Thinking".into(), now)
                        }
                    }
                }
                s.release_agent_wait(&id, now);
            }
            None if s.state == State::Waiting => match outcome {
                ApprovalOutcome::Allowed => {
                    let task = s.working_task();
                    s.enter(State::Working, task, now);
                }
                ApprovalOutcome::Denied => s.enter(State::Thinking, "Thinking".into(), now),
                ApprovalOutcome::Abandoned => s.enter(State::Idle, "Idle".into(), now),
            },
            None => {}
        },
        Event::SessionFile { .. }
        | Event::Statusline { .. }
        | Event::TranscriptModel { .. }
        | Event::TranscriptTitle { .. }
        | Event::TranscriptNext { .. }
        | Event::Shell { .. }
        | Event::Skill { .. }
        | Event::TestRun { .. }
        | Event::ToolFailed { .. }
        | Event::TranscriptCompacted
        | Event::Edited { .. }
        | Event::ShellProbe { .. }
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
    const SUB_STOP: Event = Event::SubagentStop { agent_id: None, result: None };

    fn fresh() -> Session {
        Session::new("s1", 42, "agentview", "F:/p", "interactive", "cli", T0)
    }

    fn pre(tool: &str, target: Option<&str>) -> Event {
        Event::PreToolUse { tool: tool.into(), target: target.map(Into::into), description: None }
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
            Some(LastTool { name: "Edit".into(), target: Some("src/main.ts".into()), description: None })
        );
    }

    #[test]
    fn bash_description_rides_on_last_tool_and_serialises() {
        let ev = Event::PreToolUse {
            tool: "Bash".into(),
            target: Some("npm test -- --run".into()),
            description: Some("Run the test suite".into()),
        };
        let s = apply(fresh(), ev, T0 + 1);
        let lt = s.last_tool.as_ref().unwrap();
        assert_eq!(lt.description.as_deref(), Some("Run the test suite"));
        assert_eq!(s.task, "Bash npm test -- --run");
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["lastTool"]["description"], "Run the test suite");
    }

    #[test]
    fn pre_tool_use_without_target_shows_tool_only() {
        let s = run(vec![(pre("WebSearch", None), T0 + 1)]);
        assert_eq!(s.task, "WebSearch");
    }

    #[test]
    fn working_target_truncated_to_60_chars() {
        let long = format!("{}{}", "x".repeat(50), "y".repeat(50));
        let s = run(vec![(pre("Bash", Some(&long)), T0 + 1)]);
        let target = s.last_tool.as_ref().unwrap().target.as_ref().unwrap();
        assert_eq!(target.chars().count(), 60);
        assert_eq!(*target, format!("{}…{}", "x".repeat(30), "y".repeat(29)));
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
    fn trailing_permission_notification_keeps_permission_request_text() {
        let s = run(vec![
            (pre("Write", Some("perm.txt")), T0 + 1),
            (notif("Needs permission: Write perm.txt", false), T0 + 2),
            (notif("Claude needs your permission", false), T0 + 8),
        ]);
        assert_eq!(s.state, State::Waiting);
        assert_eq!(s.task, "Needs permission: Write perm.txt");
        assert_eq!(s.state_since, T0 + 2);
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
            (SUB_STOP, T0 + 5 * MIN),
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
            (Event::Statusline { model: None, context_pct: Some(10.0), tokens: None, cache_ttl_ms: None }, T0 + 4 * MIN),
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
        let s = apply(stuck.clone(), SUB_STOP, T0 + 8 * MIN);
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
        for e in [SUB_STOP, Event::Tick, notif("idle", true), Event::SessionFile { busy: true }] {
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
            SUB_STOP,
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
        let s = apply(s, SUB_STOP, T0 + 4);
        assert_eq!(s.subagents, 1);
        let s = apply(apply(s, SUB_STOP, T0 + 5), SUB_STOP, T0 + 6);
        assert_eq!(s.subagents, 0);
        let s = apply(apply(s, pre("Task", None), T0 + 7), pre("Task", None), T0 + 8);
        assert_eq!(s.subagents, 2);
        let s = apply(s, Event::Stop, T0 + 9);
        assert_eq!(s.subagents, 0);
    }

    // --- per-agent tracking (§11.6) ---

    fn start(id: &str) -> Event {
        Event::SubagentStart { agent_id: id.into(), agent_type: "Explore".into() }
    }

    fn sub(id: &str, e: Event) -> Event {
        Event::Subagent { agent_id: id.into(), agent_type: "Explore".into(), event: Box::new(e) }
    }

    fn sub_stop(id: &str, result: &str) -> Event {
        Event::SubagentStop { agent_id: Some(id.into()), result: Some(result.into()) }
    }

    #[test]
    fn agent_lifecycle_tracks_state_tools_and_result() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (pre("Agent", Some("find endpoints")), T0 + 1),
            (start("a1"), T0 + 2),
        ]);
        let a = &s.agents[0];
        assert_eq!((a.id.as_str(), a.agent_type.as_str(), a.description.as_str()), ("a1", "Explore", "find endpoints"));
        assert_eq!((a.state, a.task.as_str(), a.started_at, a.tool_count), (State::Thinking, "Thinking", T0 + 2, 0));
        assert!(s.pending_agents.is_empty());

        let s = apply(s, sub("a1", pre("Grep", Some("route"))), T0 + 3);
        assert_eq!((s.agents[0].state, s.agents[0].task.as_str()), (State::Working, "Grep route"));
        let s = apply(s, sub("a1", Event::PostToolUse), T0 + 4);
        assert_eq!(s.agents[0].state, State::Thinking);
        let s = apply(s, sub_stop("a1", "Found 12 endpoints"), T0 + 5);
        let a = &s.agents[0];
        assert_eq!((a.state, a.task.as_str(), a.ended_at, a.tool_count), (State::Success, "Done", Some(T0 + 5), 1));
        assert_eq!(a.result.as_deref(), Some("Found 12 endpoints"));
        assert_eq!(s.subagents, 0);
    }

    #[test]
    fn subagent_tool_events_do_not_touch_parent_task_or_tool() {
        let s = run(vec![
            (pre("Agent", Some("explore")), T0),
            (start("a1"), T0 + 1),
            (sub("a1", pre("Read", Some("x.rs"))), T0 + 2),
            (sub("a1", Event::PostToolUse), T0 + 3),
        ]);
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Agent explore");
        assert_eq!(s.last_tool.as_ref().unwrap().name, "Agent");
        assert_eq!(s.state_since, T0);
    }

    #[test]
    fn subagent_events_keep_parent_from_going_stuck() {
        let s = run(vec![
            (pre("Agent", Some("long")), T0),
            (start("a1"), T0 + 1),
            (sub("a1", pre("Bash", Some("make"))), T0 + 5 * MIN),
            (Event::Tick, T0 + 7 * MIN),
        ]);
        assert_eq!(s.state, State::Working);
        let s = apply(s, Event::Tick, T0 + 11 * MIN);
        assert_eq!(s.state, State::Stuck);
        assert_eq!(s.agents[0].state, State::Stuck);
        assert_eq!(s.agents[0].task, "No response for 6 min");
        let s = apply(s, sub("a1", Event::PostToolUse), T0 + 12 * MIN);
        assert_eq!(s.agents[0].state, State::Thinking);
        assert_eq!(s.state, State::Working, "parent returns to its pre-stuck state");
    }

    #[test]
    fn subagent_permission_puts_parent_in_waiting_until_that_agent_moves_on() {
        let s = run(vec![
            (pre("Agent", Some("write it")), T0),
            (start("a1"), T0 + 1),
            (start("a2"), T0 + 1),
            (sub("a1", pre("Write", Some("f.txt"))), T0 + 2),
            (sub("a1", notif("Needs permission: Write f.txt", false)), T0 + 3),
        ]);
        assert_eq!((s.state, s.task.as_str()), (State::Waiting, "Needs permission: Write f.txt"));
        assert_eq!(s.agents[0].state, State::Waiting);
        let s = apply(s, notif("Claude needs your permission", false), T0 + 4);
        assert_eq!(s.task, "Needs permission: Write f.txt");
        let s = apply(s, sub("a2", pre("Read", Some("y"))), T0 + 5);
        assert_eq!(s.state, State::Waiting, "another agent's tool does not release it");
        let s = apply(s, sub("a1", Event::PostToolUse), T0 + 6);
        assert_eq!((s.state, s.task.as_str(), s.state_since), (State::Working, "Agent write it", T0 + 6));
        assert_eq!(s.agents[0].state, State::Thinking);
    }

    #[test]
    fn background_agent_permission_after_stop_returns_parent_to_success() {
        let s = run(vec![
            (pre("Agent", Some("bg")), T0),
            (start("a1"), T0 + 1),
            (Event::PostToolUse, T0 + 2),
            (Event::Stop, T0 + 3),
            (sub("a1", notif("Needs permission: Bash rm", false)), T0 + 4),
        ]);
        assert_eq!(s.state, State::Waiting);
        let s = apply(s, sub_stop("a1", "done"), T0 + 5);
        assert_eq!((s.state, s.task.as_str()), (State::Success, "Done"));
    }

    #[test]
    fn background_agent_outlives_stop() {
        let s = run(vec![
            (pre("Agent", Some("bg")), T0),
            (start("a1"), T0 + 1),
            (Event::PostToolUse, T0 + 2),
            (Event::Stop, T0 + 3),
            (sub("a1", pre("Read", Some("b.txt"))), T0 + 4),
        ]);
        assert_eq!(s.state, State::Success);
        assert_eq!((s.agents[0].state, s.agents[0].task.as_str()), (State::Working, "Read b.txt"));
        assert!(s.agents[0].is_busy());
        let s = apply(s, sub_stop("a1", "banana"), T0 + 5);
        assert_eq!(s.agents[0].result.as_deref(), Some("banana"));
        assert_eq!(s.state, State::Success);
    }

    #[test]
    fn agent_without_subagent_start_is_created_by_its_first_event() {
        let s = run(vec![
            (pre("Agent", Some("first")), T0),
            (pre("Agent", Some("second")), T0 + 1),
            (sub("a2", pre("Read", Some("x"))), T0 + 2),
        ]);
        assert_eq!(s.agents.len(), 1);
        assert_eq!((s.agents[0].id.as_str(), s.agents[0].description.as_str()), ("a2", "first"));
        assert_eq!(s.agents[0].started_at, T0 + 2);
        assert_eq!(s.pending_agents, vec!["second".to_string()]);
        assert!(run(vec![(pre("Agent", None), T0), (Event::Stop, T0 + 1)]).pending_agents.is_empty());
    }

    #[test]
    fn unknown_or_internal_subagent_stop_creates_no_agent() {
        let s = run(vec![(Event::Stop, T0), (SUB_STOP, T0 + 1), (sub_stop("zz", "x"), T0 + 2)]);
        assert!(s.agents.is_empty());
        assert_eq!(s.state, State::Success);
    }

    #[test]
    fn handback_report_wins_over_closing_text() {
        let s = run(vec![
            (start("a1"), T0),
            (sub("a1", pre("SubagentHandback", Some("Found 3 issues"))), T0 + 1),
            (sub_stop("a1", "Handed back."), T0 + 2),
        ]);
        assert_eq!(s.agents[0].result.as_deref(), Some("Found 3 issues"));
        assert_eq!(s.agents[0].tool_count, 1);
    }

    #[test]
    fn resumed_agent_restarts_keeping_its_description() {
        let s = run(vec![
            (pre("Agent", Some("review")), T0),
            (start("a1"), T0 + 1),
            (sub_stop("a1", "ok"), T0 + 2),
            (start("a1"), T0 + 3),
        ]);
        let a = &s.agents[0];
        assert_eq!(s.agents.len(), 1);
        assert_eq!((a.state, a.ended_at, a.result.as_deref(), a.description.as_str()), (State::Thinking, None, None, "review"));
        assert_eq!(a.started_at, T0 + 3);
    }

    #[test]
    fn events_for_finished_agents_are_ignored() {
        let s = run(vec![(start("a1"), T0), (sub_stop("a1", "ok"), T0 + 1), (sub("a1", pre("Read", None)), T0 + 2)]);
        assert_eq!((s.agents[0].state, s.agents[0].tool_count), (State::Success, 0));
    }

    #[test]
    fn finished_agents_are_capped_oldest_first() {
        let mut s = fresh();
        for i in 0..FINISHED_AGENTS_KEPT + 3 {
            let id = format!("a{i}");
            s = apply(apply(s, start(&id), T0 + i as u64), sub_stop(&id, "ok"), T0 + i as u64);
        }
        s = apply(s, start("live"), T0 + 100);
        assert_eq!(s.agents.len(), FINISHED_AGENTS_KEPT + 1);
        assert_eq!(s.agents[0].id, "a3");
        assert_eq!(s.agents.last().unwrap().id, "live");
    }

    #[test]
    fn session_end_ends_running_agents() {
        let s = run(vec![(start("a1"), T0), (start("a2"), T0), (sub_stop("a2", "ok"), T0 + 1), (Event::Ended, T0 + 2)]);
        assert_eq!((s.agents[0].state, s.agents[0].ended_at), (State::Idle, Some(T0 + 2)));
        assert_eq!((s.agents[1].state, s.agents[1].ended_at), (State::Success, Some(T0 + 1)));
    }

    #[test]
    fn agents_serialise_in_snapshot_shape() {
        let s = run(vec![(pre("Agent", Some("explore")), T0), (start("a1"), T0 + 1), (sub("a1", pre("Read", Some("a"))), T0 + 2)]);
        let v = serde_json::to_value(&s).unwrap();
        let a = &v["agents"][0];
        assert_eq!(a["id"], "a1");
        assert_eq!(a["agentType"], "Explore");
        assert_eq!(a["description"], "explore");
        assert_eq!(a["state"], "working");
        assert_eq!(a["stateSince"], T0 + 2);
        assert_eq!(a["task"], "Read a");
        assert_eq!(a["lastTool"]["name"], "Read");
        assert_eq!(a["startedAt"], T0 + 1);
        assert_eq!(a["toolCount"], 1);
        for absent in ["endedAt", "result", "handback", "lastEventAt", "preStuck"] {
            assert!(a.get(absent).is_none(), "unexpected {absent}");
        }
        for absent in ["pendingAgents", "waitingOnAgent"] {
            assert!(v.get(absent).is_none(), "unexpected {absent}");
        }
    }

    // --- background shells (AV-29) ---

    fn shell_started(id: &str, cmd: &str) -> Event {
        Event::Shell {
            update: ShellEvent::Started { id: id.into(), command: cmd.into(), description: format!("run {cmd}") },
            event: Box::new(Event::PostToolUse),
        }
    }

    fn shell_ended(id: &str, exit: ShellExit) -> Event {
        Event::Shell { update: ShellEvent::Ended { id: id.into(), exit }, event: Box::new(Event::UserPromptSubmit) }
    }

    fn probe(id: &str, obs: ShellObservation) -> Event {
        Event::ShellProbe { id: id.into(), obs }
    }

    #[test]
    fn background_bash_starts_a_running_shell_and_still_applies_post_tool_use() {
        let s = run(vec![(pre("Bash", Some("npm run dev")), T0), (shell_started("b1", "npm run dev"), T0 + 1)]);
        assert_eq!(s.state, State::Thinking);
        let sh = &s.shells[0];
        assert_eq!((sh.id.as_str(), sh.command.as_str(), sh.state), ("b1", "npm run dev", ShellState::Running));
        assert_eq!((sh.started_at, sh.state_since, sh.ended_at), (T0 + 1, T0 + 1, None));
        let again = apply(s.clone(), shell_started("b1", "other"), T0 + 2);
        assert_eq!(again.shells.len(), 1);
    }

    #[test]
    fn probe_with_listening_port_is_serving_and_losing_it_returns_to_running() {
        let s = run(vec![
            (shell_started("b1", "vite"), T0),
            (probe("b1", ShellObservation { pid: Some(77), port: Some(5173), last_output_at: Some(T0), ..Default::default() }), T0 + 5_000),
        ]);
        assert_eq!((s.shells[0].state, s.shells[0].port, s.shells[0].pid), (ShellState::Serving, Some(5173), Some(77)));
        assert_eq!(s.shells[0].state_since, T0 + 5_000);
        let s = apply(s, probe("b1", ShellObservation { last_output_at: Some(T0 + 6_000), ..Default::default() }), T0 + 10_000);
        assert_eq!((s.shells[0].state, s.shells[0].port, s.shells[0].pid), (ShellState::Running, None, Some(77)));
    }

    #[test]
    fn shell_goes_quiet_after_two_minutes_without_output_and_serving_wins() {
        let quiet = |at: u64, port| ShellObservation { last_output_at: Some(at), port, ..Default::default() };
        let s = run(vec![(shell_started("b1", "tsc -w"), T0), (probe("b1", quiet(T0, None)), T0 + SHELL_QUIET_MS - 1)]);
        assert_eq!(s.shells[0].state, ShellState::Running);
        let s = apply(s, probe("b1", quiet(T0, None)), T0 + SHELL_QUIET_MS);
        assert_eq!(s.shells[0].state, ShellState::Quiet);
        let s = apply(s, probe("b1", quiet(T0, Some(3000))), T0 + SHELL_QUIET_MS + 5_000);
        assert_eq!(s.shells[0].state, ShellState::Serving);
        let s = apply(s, probe("b1", ShellObservation::default()), T0 + 10 * MIN);
        assert_eq!(s.shells[0].state, ShellState::Running, "no output file seen: quiet is never claimed");
    }

    #[test]
    fn shell_end_states_from_notifications_and_task_stop() {
        let s = run(vec![
            (shell_started("ok", "make"), T0),
            (shell_started("bad", "make test"), T0),
            (shell_started("srv", "serve"), T0),
            (shell_started("odd", "x"), T0),
            (shell_ended("ok", ShellExit::Code(0)), T0 + 1),
            (shell_ended("bad", ShellExit::Code(3)), T0 + 2),
            (shell_ended("srv", ShellExit::Killed), T0 + 3),
            (shell_ended("odd", ShellExit::Failed), T0 + 4),
        ]);
        let got: Vec<_> = s.shells.iter().map(|sh| (sh.id.as_str(), sh.state, sh.exit_code, sh.ended_at)).collect();
        assert_eq!(
            got,
            vec![
                ("ok", ShellState::Exited, Some(0), Some(T0 + 1)),
                ("bad", ShellState::Failed, Some(3), Some(T0 + 2)),
                ("srv", ShellState::Exited, None, Some(T0 + 3)),
                ("odd", ShellState::Failed, None, Some(T0 + 4)),
            ]
        );
        assert_eq!(s.state, State::Thinking, "the notification prompt still starts a turn");
        let again = apply(s.clone(), shell_ended("bad", ShellExit::Code(0)), T0 + 9);
        assert_eq!(again.shells, s.shells, "an ended shell stays ended");
    }

    #[test]
    fn probe_trailer_ends_a_shell_and_a_later_probe_fills_the_last_line_once() {
        let s = run(vec![
            (shell_started("b1", "cargo build"), T0),
            (probe("b1", ShellObservation { exit: Some(ShellExit::Code(101)), last_line: Some("error: aborting".into()), ..Default::default() }), T0 + 5),
        ]);
        let sh = &s.shells[0];
        assert_eq!((sh.state, sh.exit_code, sh.last_line.as_deref(), sh.tail_read), (ShellState::Failed, Some(101), Some("error: aborting"), true));

        let s = run(vec![(shell_started("b2", "npm test"), T0), (shell_ended("b2", ShellExit::Code(1)), T0 + 1)]);
        assert!(!s.shells[0].tail_read);
        let s = apply(s, probe("b2", ShellObservation { last_line: Some("1 failing".into()), port: Some(1), ..Default::default() }), T0 + 2);
        let sh = &s.shells[0];
        assert_eq!((sh.state, sh.last_line.as_deref(), sh.port, sh.tail_read), (ShellState::Failed, Some("1 failing"), None, true));
    }

    #[test]
    fn shell_probes_never_reset_the_stuck_timer_or_hook_freshness() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (shell_started("b1", "x"), T0),
            (probe("b1", ShellObservation { port: Some(80), ..Default::default() }), T0 + 5 * MIN),
            (Event::Tick, T0 + 6 * MIN),
        ]);
        assert_eq!(s.state, State::Stuck);
        assert_eq!(s.last_hook_at, Some(T0));
        assert_eq!(apply(fresh(), probe("nope", ShellObservation::default()), T0), fresh());
    }

    #[test]
    fn stop_listing_adopts_unknown_shells_without_ending_known_ones() {
        let listed = |ids: &[&str]| Event::Shell {
            update: ShellEvent::Listed { shells: ids.iter().map(|i| (i.to_string(), format!("cmd {i}"), String::new())).collect() },
            event: Box::new(Event::Stop),
        };
        let s = run(vec![(shell_started("b1", "a"), T0), (listed(&["b2"]), T0 + 1)]);
        assert_eq!(s.state, State::Success);
        let got: Vec<_> = s.shells.iter().map(|sh| (sh.id.as_str(), sh.command.as_str(), sh.state)).collect();
        assert_eq!(got, vec![("b1", "a", ShellState::Running), ("b2", "cmd b2", ShellState::Running)]);
    }

    #[test]
    fn shells_end_with_the_session() {
        let s = run(vec![
            (shell_started("b1", "serve"), T0),
            (shell_started("b2", "make"), T0),
            (shell_ended("b2", ShellExit::Code(2)), T0 + 1),
            (Event::Ended, T0 + 2),
        ]);
        assert_eq!((s.shells[0].state, s.shells[0].exit_code, s.shells[0].ended_at), (ShellState::Exited, None, Some(T0 + 2)));
        assert_eq!((s.shells[1].state, s.shells[1].exit_code), (ShellState::Failed, Some(2)));
        let after = apply(s.clone(), shell_started("b3", "late"), T0 + 3);
        assert_eq!(after, s);
    }

    #[test]
    fn finished_shells_are_capped_oldest_first() {
        let mut s = fresh();
        for i in 0..FINISHED_SHELLS_KEPT + 2 {
            let id = format!("b{i}");
            s = apply(apply(s, shell_started(&id, "x"), T0), shell_ended(&id, ShellExit::Code(0)), T0);
        }
        s = apply(s, shell_started("live", "x"), T0);
        assert_eq!(s.shells.len(), FINISHED_SHELLS_KEPT + 1);
        assert_eq!(s.shells[0].id, "b2");
    }

    #[test]
    fn shells_serialise_in_snapshot_shape() {
        let s = run(vec![
            (shell_started("b1", "vite"), T0),
            (probe("b1", ShellObservation { pid: Some(9), port: Some(5173), last_line: Some("ready".into()), ..Default::default() }), T0 + 1),
        ]);
        let v = serde_json::to_value(&s).unwrap();
        let sh = &v["shells"][0];
        assert_eq!(sh["id"], "b1");
        assert_eq!(sh["command"], "vite");
        assert_eq!(sh["description"], "run vite");
        assert_eq!(sh["state"], "serving");
        assert_eq!(sh["stateSince"], T0 + 1);
        assert_eq!(sh["startedAt"], T0);
        assert_eq!(sh["port"], 5173);
        assert_eq!(sh["lastLine"], "ready");
        for absent in ["endedAt", "exitCode", "pid", "tailRead"] {
            assert!(sh.get(absent).is_none(), "unexpected {absent}");
        }
        let states: Vec<_> = [ShellState::Running, ShellState::Quiet, ShellState::Serving, ShellState::Exited, ShellState::Failed]
            .iter()
            .map(|st| serde_json::to_value(st).unwrap())
            .collect();
        assert_eq!(states, ["running", "quiet", "serving", "exited", "failed"]);
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

    #[test]
    fn transcript_tool_use_never_counts_subagents() {
        let s = run(vec![(Event::TranscriptToolUse { tool: "Agent".into(), target: Some("x".into()) }, T0 + 1)]);
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Agent x");
        assert_eq!(s.subagents, 0);
        let s = apply(s, pre("Agent", Some("x")), T0 + 2);
        assert_eq!(s.subagents, 1);
    }

    #[test]
    fn transcript_model_fills_model_without_state_change_or_timer_reset() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (Event::TranscriptModel { model: "Opus 5.5".into() }, T0 + 5 * MIN),
            (Event::Tick, T0 + 6 * MIN),
        ]);
        assert_eq!(s.model.as_deref(), Some("Opus 5.5"));
        assert_eq!(s.state, State::Stuck);
        let s = apply(fresh(), Event::TranscriptModel { model: "Sonnet 4.5".into() }, T0 + 1);
        let s = apply(s, Event::TranscriptModel { model: "Opus 5.5".into() }, T0 + 2);
        assert_eq!(s.model.as_deref(), Some("Opus 5.5"));
        assert_eq!(s.state, State::Idle);
        assert_eq!(s.last_hook_at, None);
    }

    #[test]
    fn statusline_model_wins_over_transcript_model() {
        let s = run(vec![
            (Event::TranscriptModel { model: "Opus 5.5".into() }, T0),
            (Event::Statusline { model: Some("Opus 5.5 (1M context)".into()), context_pct: None, tokens: None, cache_ttl_ms: None }, T0 + 1),
            (Event::TranscriptModel { model: "Opus 5.5".into() }, T0 + 2),
        ]);
        assert_eq!(s.model.as_deref(), Some("Opus 5.5 (1M context)"));
    }

    // --- metadata, lifecycle, timestamps ---

    #[test]
    fn statusline_fills_model_and_context_without_state_change() {
        let s = run(vec![
            (Event::UserPromptSubmit, T0),
            (Event::Statusline { model: Some("Opus".into()), context_pct: Some(42.5), tokens: None, cache_ttl_ms: None }, T0 + 1),
            (Event::Statusline { model: None, context_pct: None, tokens: None, cache_ttl_ms: None }, T0 + 2),
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
            (Event::Statusline { model: None, context_pct: Some(12.0), tokens: None, cache_ttl_ms: None }, T0 + 2),
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

    // --- skill and ticket link (§13.2) ---

    fn ticket(id: &str, title: Option<&str>) -> Option<TicketLink> {
        Some(TicketLink { id: id.into(), title: title.map(Into::into) })
    }

    fn skill(name: &str, t: Option<TicketLink>, e: Event) -> Event {
        Event::Skill { skill: name.into(), ticket: t, event: Box::new(e) }
    }

    #[test]
    fn slash_prompt_links_skill_and_ticket_and_serialises() {
        let s = run(vec![(skill("ticket", ticket("AV-64", Some("Link it")), Event::UserPromptSubmit), T0 + 1)]);
        assert_eq!(s.state, State::Thinking);
        assert_eq!((s.skill.as_deref(), s.ticket.clone()), (Some("ticket"), ticket("AV-64", Some("Link it"))));
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["skill"], "ticket");
        assert_eq!(v["ticket"], serde_json::json!({ "id": "AV-64", "title": "Link it" }));
        assert!(serde_json::to_value(fresh()).unwrap().get("ticket").is_none());
    }

    #[test]
    fn skill_tool_pre_tool_use_links_and_keeps_ticket_without_id() {
        let s = run(vec![
            (skill("ticket", ticket("AV-7", None), Event::UserPromptSubmit), T0 + 1),
            (skill("shoot", None, pre("Skill", Some("peep:shoot"))), T0 + 2),
        ]);
        assert_eq!(s.state, State::Working);
        assert_eq!((s.skill.as_deref(), s.ticket.clone()), (Some("shoot"), ticket("AV-7", None)));
        let s = apply(s, skill("ticket", ticket("AV-8", None), pre("Skill", None)), T0 + 3);
        assert_eq!(s.ticket, ticket("AV-8", None));
    }

    #[test]
    fn stop_ejects_skill_and_ticket_survives_turns_until_next_peep_prompt() {
        let s = run(vec![
            (skill("ticket", ticket("AV-64", None), Event::UserPromptSubmit), T0 + 1),
            (Event::Stop, T0 + 2),
        ]);
        assert_eq!((s.skill.clone(), s.ticket.clone()), (None, ticket("AV-64", None)));
        let s = apply(apply(s, Event::UserPromptSubmit, T0 + 3), Event::Stop, T0 + 4);
        assert_eq!(s.ticket, ticket("AV-64", None), "plain turns keep the ticket");
        let s = apply(s, skill("packup", None, Event::UserPromptSubmit), T0 + 5);
        assert_eq!((s.skill.as_deref(), s.ticket.clone()), (Some("packup"), None));
        let s = apply(s, Event::Ended, T0 + 6);
        assert_eq!((s.skill.clone(), s.ticket.clone()), (None, None));
    }

    #[test]
    fn unknown_peep_skill_links_nothing() {
        let s = run(vec![
            (skill("ticket", ticket("AV-1", None), Event::UserPromptSubmit), T0 + 1),
            (skill("bogus", ticket("AV-2", None), pre("Skill", None)), T0 + 2),
        ]);
        assert_eq!((s.skill.as_deref(), s.ticket.clone()), (Some("ticket"), ticket("AV-1", None)));
        assert_eq!(s.state, State::Working, "the wrapped event still applies");
    }

    #[test]
    fn every_peep_skill_seats_its_floppy() {
        for name in PEEP_SKILLS {
            let s = run(vec![(skill(name, None, Event::UserPromptSubmit), T0 + 1)]);
            assert_eq!(s.skill.as_deref(), Some(name));
        }
    }

    #[test]
    fn next_line_needs_a_peep_skill_and_a_new_run_clears_it() {
        let next = |l: &str| Event::TranscriptNext { line: l.into() };
        let s = run(vec![(Event::UserPromptSubmit, T0 + 1), (next("/peep:ticket AV-1"), T0 + 2)]);
        assert_eq!(s.next, None, "no Peep skill has run");

        // The text line can land after Stop; a plain prompt in between (a read-back answer) keeps the run's line.
        let s = run(vec![
            (skill("ducks", None, Event::UserPromptSubmit), T0 + 1),
            (Event::Stop, T0 + 2),
            (Event::UserPromptSubmit, T0 + 3),
            (next("/peep:ticket AV-12"), T0 + 4),
            (Event::Stop, T0 + 5),
        ]);
        assert_eq!(s.next.as_deref(), Some("/peep:ticket AV-12"));
        assert_eq!(s.last_event_at, T0 + 5, "a next line is not activity");
        assert_eq!(serde_json::to_value(&s).unwrap()["next"], "/peep:ticket AV-12");

        let s = apply(s, Event::UserPromptSubmit, T0 + 6);
        assert_eq!(s.next.as_deref(), Some("/peep:ticket AV-12"), "kept across turns");
        let s = apply(s, skill("packup", None, Event::UserPromptSubmit), T0 + 7);
        assert_eq!(s.next, None, "a new Peep skill run starts clean");
    }

    // --- AV-109 signal tips ---

    fn usage(pct: Option<f64>, tokens: Option<u64>) -> Event {
        Event::Statusline { model: None, context_pct: pct, tokens, cache_ttl_ms: None }
    }

    fn failed(tool: &str) -> Event {
        Event::ToolFailed { tool: tool.into(), event: Box::new(Event::PostToolUse) }
    }

    #[test]
    fn crowded_raises_once_per_session_on_tokens_or_percent_fallback() {
        let s = run(vec![(Event::UserPromptSubmit, T0 + 1), (usage(Some(12.0), Some(119_000)), T0 + 2)]);
        assert_eq!(s.signal, None, "12% of a 1M window and under 120k");
        let s = apply(s, usage(Some(12.0), Some(120_000)), T0 + 3);
        assert_eq!(s.signal, Some(Tip::Crowded));
        let mut s = s;
        s.signal = None;
        assert_eq!(apply(s, usage(Some(13.0), Some(130_000)), T0 + 4).signal, None, "once per session");
        assert_eq!(run(vec![(usage(Some(60.0), None), T0 + 1)]).signal, Some(Tip::Crowded), "no tokens: % of window");
    }

    #[test]
    fn same_tool_failing_twice_in_a_turn_raises_misses() {
        let s = run(vec![(Event::UserPromptSubmit, T0 + 1), (failed("Bash"), T0 + 2), (failed("Edit"), T0 + 3)]);
        assert_eq!(s.signal, None);
        assert_eq!(s.state, State::Thinking, "the wrapped PostToolUse still applies");
        assert_eq!(apply(s.clone(), failed("Bash"), T0 + 4).signal, Some(Tip::Misses));
        let s = apply(s, Event::UserPromptSubmit, T0 + 4);
        assert_eq!(apply(s, failed("Bash"), T0 + 5).signal, None, "a prompt resets the count");
    }

    #[test]
    fn idle_past_the_cache_lifetime_raises_cold_once_per_stretch() {
        let s = run(vec![(Event::UserPromptSubmit, T0 + 1), (usage(Some(10.0), Some(20_000)), T0 + 2), (Event::Stop, T0 + 3)]);
        let ttl = tip::CACHE_TTL_DEFAULT_MS;
        assert!(!s.cold_due(T0 + 3 + ttl - 1));
        assert!(s.cold_due(T0 + 3 + ttl));
        let s = apply(s, Event::Tick, T0 + 3 + ttl);
        assert_eq!(s.signal, Some(Tip::Cold));
        assert!(!s.cold_due(T0 + 3 + 2 * ttl), "once per idle stretch");

        let short = run(vec![
            (Event::Statusline { model: None, context_pct: Some(5.0), tokens: Some(9_000), cache_ttl_ms: tip::ttl_ms("5m") }, T0 + 1),
            (Event::Stop, T0 + 2),
        ]);
        assert!(short.cold_due(T0 + 2 + 5 * MIN), "an API-key session's 5m cache");
        assert!(!fresh().cold_due(T0 + ttl), "no request yet, no cache to go cold");
        let busy = run(vec![(Event::UserPromptSubmit, T0 + 1), (usage(Some(10.0), None), T0 + 2)]);
        assert!(!busy.cold_due(T0 + 2 + ttl), "busy sessions go stuck, not cold");
    }

    #[test]
    fn first_subagent_and_auto_compact_raise_their_tips() {
        let s = run(vec![(Event::SubagentStart { agent_id: "a1".into(), agent_type: "Explore".into() }, T0 + 1)]);
        assert_eq!(s.signal, Some(Tip::Helper));
        assert_eq!(run(vec![(Event::TranscriptCompacted, T0 + 1)]).signal, Some(Tip::Compacted));
    }

    #[test]
    fn a_prompt_clears_the_tip_on_show() {
        let mut s = run(vec![(Event::Stop, T0 + 1)]);
        s.tip = Some(Tip::Helper);
        assert_eq!(apply(s, Event::UserPromptSubmit, T0 + 2).tip, None);
    }
}
