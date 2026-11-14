pub mod claude;
pub mod process;

/// Agent-neutral identity of a session; adapters translate their own formats into this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub pid: u32,
    pub name: String,
    pub cwd: String,
    pub kind: String,
    pub entrypoint: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceEvent {
    /// Session is live. `busy` is `None` when the source does not know.
    Upsert { info: SessionInfo, busy: Option<bool> },
    Ended { id: String },
    /// A state-engine event for a session the source already reported.
    Apply { id: String, event: crate::state::Event },
}

pub type Sink = Box<dyn Fn(SourceEvent) + Send>;

/// One background shell for the OS probe to look at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellQuery {
    pub session_id: String,
    /// 0 for a placeholder session whose session file has not been seen.
    pub session_pid: u32,
    pub shell_id: String,
    pub command: String,
    /// PID matched by an earlier probe.
    pub pid: Option<u32>,
    pub ended: bool,
}

pub trait AgentSource: Send + 'static {
    /// One-shot scan of the sessions that exist right now.
    fn discover(&mut self) -> Vec<SourceEvent>;

    /// Blocks forever, pushing every change to `sink`.
    fn events(self: Box<Self>, sink: Sink);
}

/// Runs each source's initial scan and then its watcher on its own thread.
pub fn spawn_all(sources: Vec<Box<dyn AgentSource>>, make_sink: impl Fn() -> Sink) {
    for mut src in sources {
        let sink = make_sink();
        for ev in src.discover() {
            sink(ev);
        }
        std::thread::Builder::new()
            .name("agent-source".into())
            .spawn(move || src.events(sink))
            .expect("spawn source thread");
    }
}
