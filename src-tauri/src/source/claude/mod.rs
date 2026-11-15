pub mod sessions;
pub mod shells;
pub mod skill;
pub mod transcript;

use std::path::PathBuf;
use std::sync::mpsc;

use super::{AgentSource, Sink, SourceEvent};
use sessions::SessionDir;
use transcript::Msg;

pub struct ClaudeSource {
    sessions: SessionDir,
    projects: PathBuf,
    /// Sessions reported by `discover`, handed to the transcript tailer when `events` starts.
    discovered: Vec<(String, String)>,
}

impl ClaudeSource {
    pub fn new() -> Self {
        let dir = config_dir();
        ClaudeSource {
            sessions: SessionDir::new(dir.join("sessions")),
            projects: dir.join("projects"),
            discovered: Vec::new(),
        }
    }
}

impl AgentSource for ClaudeSource {
    fn discover(&mut self) -> Vec<SourceEvent> {
        let events = self.sessions.scan();
        self.discovered = events.iter().filter_map(track_key).collect();
        events
    }

    /// Session watcher and transcript tailer each run on their own thread; this thread forwards
    /// both to `sink`, telling the tailer about a session only after its `Upsert` is sunk.
    fn events(self: Box<Self>, sink: Sink) {
        let ClaudeSource { sessions, projects, discovered } = *self;
        let (ev_tx, ev_rx) = mpsc::channel::<SourceEvent>();
        let (tr_tx, tr_rx) = mpsc::channel::<Msg>();
        for (id, cwd) in discovered {
            let _ = tr_tx.send(Msg::Track { id, cwd });
        }

        let session_tx = ev_tx.clone();
        std::thread::Builder::new()
            .name("claude-sessions".into())
            .spawn(move || {
                sessions.watch(Box::new(move |ev| {
                    let _ = session_tx.send(ev);
                }))
            })
            .expect("spawn claude sessions watcher");

        let fs_tx = tr_tx.clone();
        std::thread::Builder::new()
            .name("claude-transcripts".into())
            .spawn(move || {
                transcript::run(projects, fs_tx, tr_rx, move |ev| {
                    let _ = ev_tx.send(ev);
                })
            })
            .expect("spawn claude transcript tailer");

        for ev in ev_rx {
            let ctl = match &ev {
                SourceEvent::Ended { id } => Some(Msg::Untrack { id: id.clone() }),
                _ => track_key(&ev).map(|(id, cwd)| Msg::Track { id, cwd }),
            };
            sink(ev);
            if let Some(ctl) = ctl {
                let _ = tr_tx.send(ctl);
            }
        }
    }
}

fn track_key(ev: &SourceEvent) -> Option<(String, String)> {
    match ev {
        SourceEvent::Upsert { info, .. } => Some((info.id.clone(), info.cwd.clone())),
        _ => None,
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
