//! Closed shelf record (AV-80): ended sessions in `<dir>/closed.json`, kept 8h across restarts.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::state::{Session, State, TicketLink};
use crate::tickets::{self, Repo, Status};

pub const KEEP_MS: u64 = 8 * 60 * 60 * 1000;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    pub uncommitted: bool,
    pub ticket_doing: bool,
    pub no_packup: bool,
    /// Ended in `waiting`, `error` or `stuck`.
    pub attention: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub session_id: String,
    pub name: String,
    pub cwd: String,
    pub bubble: String,
    pub state: State,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<TicketLink>,
    pub ended_at: u64,
    pub flags: Flags,
}

/// What the store hands over at the moment a session ends: the session as it was just before.
#[derive(Debug, Clone)]
pub struct Ended {
    pub session: Session,
    pub ticket_doing: bool,
    pub at: u64,
}

impl Ended {
    pub fn new(before: &Session, repos: &[Repo], at: u64) -> Ended {
        let ticket_doing = before.ticket.as_ref().is_some_and(|t| {
            repos
                .iter()
                .filter(|r| r.sessions.contains(&before.id))
                .flat_map(|r| &r.tickets)
                .any(|x| x.id == t.id && x.status == Status::Doing)
        });
        Ended { session: before.clone(), ticket_doing, at }
    }
}

/// Adds the flags that need the disk: git status and the Pack Up note's mtime.
pub fn entry(e: &Ended) -> Entry {
    let s = &e.session;
    let root = tickets::repo_root(Path::new(&s.cwd));
    Entry {
        session_id: s.id.clone(),
        name: s.name.clone(),
        cwd: s.cwd.clone(),
        bubble: s.task.clone(),
        state: s.state,
        ticket: s.ticket.clone(),
        ended_at: e.at,
        flags: Flags {
            uncommitted: !s.cwd.is_empty() && uncommitted(Path::new(&s.cwd)),
            ticket_doing: e.ticket_doing,
            no_packup: !packed_up(&root, s.born_at),
            attention: matches!(s.state, State::Waiting | State::Error | State::Stuck),
        },
    }
}

fn uncommitted(cwd: &Path) -> bool {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(cwd).args(["status", "--porcelain"]);
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut cmd, 0x0800_0000); // CREATE_NO_WINDOW
    cmd.output().is_ok_and(|o| o.status.success() && !o.stdout.is_empty())
}

/// True when `.peep/handoff.md` was written at or after `since` (ms).
fn packed_up(root: &Path, since: u64) -> bool {
    fs::metadata(root.join(".peep").join("handoff.md"))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .is_some_and(|d| d.as_millis() as u64 >= since)
}

fn prune(entries: &mut Vec<Entry>, now: u64) -> bool {
    let before = entries.len();
    entries.retain(|e| now.saturating_sub(e.ended_at) < KEEP_MS);
    entries.len() != before
}

pub struct Shelf {
    path: PathBuf,
    entries: Mutex<Vec<Entry>>,
}

impl Shelf {
    /// Loads `<dir>/closed.json`, dropping entries older than `KEEP_MS`; a missing or bad file is empty.
    pub fn open(dir: &Path, now: u64) -> Shelf {
        let path = dir.join("closed.json");
        let mut entries: Vec<Entry> =
            fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let shelf = Shelf { path, entries: Mutex::new(Vec::new()) };
        if prune(&mut entries, now) {
            if let Err(e) = shelf.write(&entries) {
                eprintln!("peepthis: closed shelf write failed: {e}");
            }
        }
        *shelf.entries.lock().unwrap() = entries;
        shelf
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.entries.lock().unwrap().clone()
    }

    /// Replaces any entry for the same session, prunes, writes; returns the new list, newest first.
    pub fn record(&self, entry: Entry, now: u64) -> io::Result<Vec<Entry>> {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|e| e.session_id != entry.session_id);
        entries.insert(0, entry);
        prune(&mut entries, now);
        self.write(&entries)?;
        Ok(entries.clone())
    }

    fn write(&self, entries: &[Entry]) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec(entries)?)?;
        fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::Ticket;

    const T0: u64 = 1_700_000_000_000;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("peepthis-closed-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn git(dir: &Path, args: &[&str]) {
        assert!(Command::new("git").arg("-C").arg(dir).args(args).output().unwrap().status.success());
    }

    fn session(cwd: &Path, state: State) -> Session {
        let mut s = Session::new("s1", 42, "demo", cwd.to_str().unwrap(), "interactive", "cli", T0);
        s.state = state;
        s.task = "Editing main.rs".into();
        s
    }

    fn ended(s: &Session) -> Ended {
        Ended { session: s.clone(), ticket_doing: false, at: T0 + 1_000 }
    }

    #[test]
    fn uncommitted_flag_follows_git_status() {
        let repo = tmp("git");
        git(&repo, &["init", "-q"]);
        let s = session(&repo, State::Idle);
        assert!(!entry(&ended(&s)).flags.uncommitted);
        fs::write(repo.join("a.txt"), "x").unwrap();
        assert!(entry(&ended(&s)).flags.uncommitted);
        let plain = tmp("plain");
        assert!(!entry(&ended(&session(&plain, State::Idle))).flags.uncommitted, "not a repo");
    }

    #[test]
    fn no_packup_flag_uses_handoff_mtime_against_session_start() {
        let repo = tmp("packup");
        fs::create_dir_all(repo.join(".peep")).unwrap();
        fs::write(repo.join(".peep").join("setup.md"), "---\nprefix: AV\n---\n").unwrap();
        let mut s = session(&repo, State::Idle);
        s.born_at = crate::store::now_ms() + 60_000;
        assert!(entry(&ended(&s)).flags.no_packup, "no note at all");
        fs::write(repo.join(".peep").join("handoff.md"), "note").unwrap();
        assert!(entry(&ended(&s)).flags.no_packup, "note older than the session");
        s.born_at = crate::store::now_ms() - 60_000;
        assert!(!entry(&ended(&s)).flags.no_packup);
    }

    #[test]
    fn attention_flag_for_waiting_error_and_stuck() {
        let dir = tmp("attention");
        for (st, want) in [(State::Waiting, true), (State::Error, true), (State::Stuck, true), (State::Success, false), (State::Idle, false)] {
            assert_eq!(entry(&ended(&session(&dir, st))).flags.attention, want, "{st:?}");
        }
    }

    #[test]
    fn ticket_doing_flag_reads_the_repo_ticket_status() {
        let dir = tmp("ticket");
        let mut s = session(&dir, State::Idle);
        s.ticket = Some(TicketLink { id: "AV-1".into(), title: None });
        let repo = |status| Repo {
            root: dir.to_str().unwrap().into(),
            sessions: vec!["s1".into()],
            tickets: vec![Ticket { id: "AV-1".into(), title: "t".into(), status, reason: None, wake: None }],
            last_closed: None,
            stale: vec![],
        };
        assert!(Ended::new(&s, &[repo(Status::Doing)], T0).ticket_doing);
        assert!(!Ended::new(&s, &[repo(Status::Done)], T0).ticket_doing);
        s.ticket = None;
        assert!(!Ended::new(&s, &[repo(Status::Doing)], T0).ticket_doing);
    }

    fn sample(id: &str, ended_at: u64) -> Entry {
        Entry {
            session_id: id.into(),
            name: id.into(),
            cwd: "C:/work/demo".into(),
            bubble: "Done".into(),
            state: State::Success,
            ticket: None,
            ended_at,
            flags: Flags::default(),
        }
    }

    #[test]
    fn prune_drops_entries_older_than_eight_hours_on_load_and_write() {
        let dir = tmp("prune");
        let shelf = Shelf::open(&dir, T0);
        shelf.record(sample("old", T0), T0).unwrap();
        let list = shelf.record(sample("new", T0 + KEEP_MS), T0 + KEEP_MS).unwrap();
        assert_eq!(list.iter().map(|e| e.session_id.as_str()).collect::<Vec<_>>(), ["new"]);

        let shelf = Shelf::open(&dir, T0 + 2 * KEEP_MS);
        assert!(shelf.entries().is_empty());
        let on_disk: Vec<Entry> = serde_json::from_slice(&fs::read(dir.join("closed.json")).unwrap()).unwrap();
        assert!(on_disk.is_empty(), "load prune is written back");
    }

    #[test]
    fn restart_round_trip_keeps_entries_and_flags() {
        let dir = tmp("restart");
        let mut e = sample("s1", T0);
        e.flags.uncommitted = true;
        e.ticket = Some(TicketLink { id: "AV-80".into(), title: Some("Ended sessions".into()) });
        Shelf::open(&dir, T0).record(e.clone(), T0).unwrap();
        Shelf::open(&dir, T0).record(sample("s2", T0 + 5), T0 + 5).unwrap();
        let reopened = Shelf::open(&dir, T0 + 10);
        assert_eq!(reopened.entries(), vec![sample("s2", T0 + 5), e]);
    }

    #[test]
    fn same_session_ending_twice_keeps_one_entry() {
        let dir = tmp("twice");
        let shelf = Shelf::open(&dir, T0);
        shelf.record(sample("s1", T0), T0).unwrap();
        assert_eq!(shelf.record(sample("s1", T0 + 1), T0 + 1).unwrap().len(), 1);
    }
}
