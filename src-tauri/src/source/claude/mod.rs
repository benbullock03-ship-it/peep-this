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
pub fn config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("CLAUDE_CONFIG_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .unwrap_or_default();
    PathBuf::from(home).join(".claude")
}

/// Contract tests: every Claude Code fixture folder (`legacy/` and each `npm run capture` version) must parse.
#[cfg(test)]
mod contract {
    use crate::ingest::server::{parse_value, Route};
    use serde_json::Value;
    use std::path::{Path, PathBuf};

    fn version_dirs() -> Vec<PathBuf> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| ["hooks", "sessions", "transcripts"].iter().any(|k| p.join(k).is_dir()))
            .collect();
        dirs.sort();
        dirs
    }

    /// Files in `dir/kind` with extension `ext`; `malformed*` fixtures exist to fail and are skipped.
    fn files(dir: &Path, kind: &str, ext: &str) -> Vec<PathBuf> {
        let Ok(rd) = std::fs::read_dir(dir.join(kind)) else { return Vec::new() };
        let mut out: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == ext))
            .filter(|p| !p.file_name().unwrap().to_string_lossy().starts_with("malformed"))
            .collect();
        out.sort();
        out
    }

    fn hook_route(stem: &str, v: &Value) -> Option<Route> {
        if stem.starts_with("statusline") {
            return Some(Route::Statusline);
        }
        v.get("hook_event_name").and_then(Value::as_str).map(|n| Route::Hook(n.to_owned()))
    }

    #[test]
    fn every_version_folder_parses() {
        let dirs = version_dirs();
        assert!(dirs.iter().any(|d| d.ends_with("legacy")), "legacy fixtures missing");
        for dir in &dirs {
            let mut parsed = 0;
            for f in files(dir, "sessions", "json") {
                let stem = f.file_stem().unwrap().to_string_lossy().into_owned();
                super::sessions::parse(&std::fs::read(&f).unwrap(), &stem)
                    .unwrap_or_else(|e| panic!("{}: {e}", f.display()));
                parsed += 1;
            }
            for f in files(dir, "hooks", "json") {
                let stem = f.file_stem().unwrap().to_string_lossy().into_owned();
                let v: Value = serde_json::from_slice(&std::fs::read(&f).unwrap()).unwrap();
                let Some(route) = hook_route(&stem, &v) else { continue };
                parse_value(&route, &v).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
                parsed += 1;
            }
            for f in files(dir, "hooks", "jsonl") {
                for line in std::fs::read_to_string(&f).unwrap().lines().filter(|l| !l.trim().is_empty()) {
                    let v: Value = serde_json::from_str(line).unwrap();
                    let route = Route::Hook(v["hook"].as_str().unwrap().to_owned());
                    parse_value(&route, &v["body"]).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
                    parsed += 1;
                }
            }
            for f in files(dir, "transcripts", "jsonl") {
                for line in std::fs::read_to_string(&f).unwrap().lines().filter(|l| !l.trim().is_empty()) {
                    super::transcript::parse_line(line, r"C:\work\demo")
                        .unwrap_or_else(|e| panic!("{}: {e}", f.display()));
                    parsed += 1;
                }
            }
            assert!(parsed > 0, "{} has no fixtures", dir.display());
        }
    }
}
