pub mod sessions;

use std::path::PathBuf;

use super::{AgentSource, Sink, SourceEvent};
use sessions::SessionDir;

pub struct ClaudeSource {
    sessions: SessionDir,
}

impl ClaudeSource {
    pub fn new() -> Self {
        ClaudeSource { sessions: SessionDir::new(config_dir().join("sessions")) }
    }
}

impl AgentSource for ClaudeSource {
    fn discover(&mut self) -> Vec<SourceEvent> {
        self.sessions.scan()
    }

    fn events(self: Box<Self>, sink: Sink) {
        self.sessions.watch(sink);
    }
}

/// Claude Code honours `CLAUDE_CONFIG_DIR` in place of `~/.claude`.
fn config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("CLAUDE_CONFIG_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .unwrap_or_default();
    PathBuf::from(home).join(".claude")
}
