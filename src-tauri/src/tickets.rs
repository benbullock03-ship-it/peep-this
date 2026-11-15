//! A repo's tickets folder: the ticket a Peep skill call links, and each live repo's shelf (spec §13.2).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use serde::Serialize;

use crate::state::TicketLink;
use crate::store::{self, Store};

const POLL: Duration = Duration::from_secs(5);
const DEBOUNCE: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Doing,
    Blocked,
    Deferred,
    Done,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wake: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Closed {
    pub id: String,
    pub at: u64,
}

/// One repo with live sessions: its open and parked tickets, read once for all its sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub root: String,
    pub sessions: Vec<String>,
    pub tickets: Vec<Ticket>,
    /// The latest ticket seen closing since the widget first read this repo (AV-70 cheer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_closed: Option<Closed>,
    /// `doing` tickets whose file has not changed for the setup's `stale.days` (AV-78 worry).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stale: Vec<String>,
}

pub const STALE_DAYS_DEFAULT: u64 = 3;
const DAY_MS: u64 = 24 * 60 * 60 * 1000;

fn stale_days(repo: &Path) -> u64 {
    setup_key(repo, "stale.days").and_then(|d| d.parse().ok()).unwrap_or(STALE_DAYS_DEFAULT)
}

fn modified_ms(path: &Path) -> Option<u64> {
    let t = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(t.duration_since(std::time::UNIX_EPOCH).ok()?.as_millis() as u64)
}

/// The first `PREFIX-N` argument, uppercased, with its title when the ticket file is found in the repo `cwd` is in.
pub fn link(args: &str, cwd: Option<&str>) -> Option<TicketLink> {
    let id = args.split_whitespace().find(|t| is_ticket_id(t))?.to_ascii_uppercase();
    let title = cwd.filter(|c| !c.is_empty()).and_then(|c| title(&repo_root(Path::new(c)), &id));
    Some(TicketLink { id, title })
}

fn is_ticket_id(t: &str) -> bool {
    let Some((prefix, n)) = t.split_once('-') else { return false };
    prefix.starts_with(|c: char| c.is_ascii_alphabetic())
        && prefix.chars().all(|c| c.is_ascii_alphanumeric())
        && !n.is_empty()
        && n.chars().all(|c| c.is_ascii_digit())
}

/// The nearest ancestor holding `.git` or a repo setup, else `cwd` itself.
pub fn repo_root(cwd: &Path) -> PathBuf {
    cwd.ancestors()
        .find(|d| d.join(".git").exists() || d.join(".peep").join("setup.md").is_file())
        .unwrap_or(cwd)
        .to_path_buf()
}

/// The repo setup's `test:` line for the repo `cwd` is in.
pub fn setup_test(cwd: &str) -> Option<String> {
    setup_key(&repo_root(Path::new(cwd)), "test")
}

fn setup_key(repo: &Path, key: &str) -> Option<String> {
    let setup = std::fs::read_to_string(repo.join(".peep").join("setup.md")).ok()?;
    frontmatter(&setup)?.get(key).filter(|v| !v.is_empty()).map(|v| v.to_string())
}

/// The repo setup's `tickets:` line (`plugin/references/formats.md` §1), else `tickets/`.
pub fn tickets_dir(repo: &Path) -> PathBuf {
    repo.join(setup_key(repo, "tickets").as_deref().unwrap_or("tickets"))
}

/// The repo setup's `archive:` line, else `<tickets>/archive/`.
pub fn archive_dir(repo: &Path) -> PathBuf {
    match setup_key(repo, "archive") {
        Some(a) => repo.join(a),
        None => tickets_dir(repo).join("archive"),
    }
}

/// Flat frontmatter keys (formats.md); `None` without an opening and closing `---`.
fn frontmatter(text: &str) -> Option<HashMap<&str, &str>> {
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut keys = HashMap::new();
    for l in lines {
        if l.trim() == "---" {
            return Some(keys);
        }
        if let Some((k, v)) = l.split_once(':') {
            keys.entry(k.trim()).or_insert(v.trim().trim_matches(['"', '\'']));
        }
    }
    None
}

fn title(repo: &Path, id: &str) -> Option<String> {
    let prefix = format!("{id}-");
    let file = md_files(&tickets_dir(repo)).into_iter().find(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_uppercase();
        name.starts_with(&prefix)
    })?;
    let text = std::fs::read_to_string(file).ok()?;
    frontmatter(&text)?.get("title").filter(|t| !t.is_empty()).map(|t| t.to_string())
}

fn md_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("md")))
        .collect();
    files.sort();
    files
}

/// `AV-65` from `AV-65-repo-tickets.md`.
fn id_from_name(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    let mut parts = stem.splitn(3, '-');
    let id = format!("{}-{}", parts.next()?, parts.next()?);
    is_ticket_id(&id).then(|| id.to_ascii_uppercase())
}

/// One ticket file; `None` when it has no frontmatter or no known status.
pub fn parse(text: &str, path: &Path) -> Option<Ticket> {
    let fm = frontmatter(text)?;
    let status = match *fm.get("status")? {
        "open" => Status::Open,
        "doing" => Status::Doing,
        "blocked" => Status::Blocked,
        "deferred" => Status::Deferred,
        "done" => Status::Done,
        "closed" => Status::Closed,
        _ => return None,
    };
    let key = |k: &str| fm.get(k).filter(|v| !v.is_empty()).map(|v| v.to_string());
    let id = key("id").map(|i| i.to_ascii_uppercase()).or_else(|| id_from_name(path))?;
    Some(Ticket { id, title: key("title").unwrap_or_default(), status, reason: key("reason"), wake: key("wake") })
}

/// Every parseable ticket directly in `dir`; malformed files are skipped.
fn read_tickets(dir: &Path) -> Vec<(PathBuf, Ticket)> {
    md_files(dir)
        .into_iter()
        .filter_map(|p| std::fs::read_to_string(&p).ok().and_then(|t| parse(&t, &p)).map(|t| (p, t)))
        .collect()
}

/// Per repo, the ids seen closed at the last read and the latest closing.
#[derive(Default)]
pub struct Shelf {
    closed: HashMap<PathBuf, (HashSet<String>, Option<Closed>)>,
}

impl Shelf {
    /// `sessions` is (id, cwd) of live sessions. A closing is an id newly in the archive or newly `closed`.
    pub fn read(&mut self, sessions: &[(String, String)], now: u64) -> Vec<Repo> {
        let mut groups: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
        for (id, cwd) in sessions.iter().filter(|(_, c)| !c.is_empty()) {
            groups.entry(repo_root(Path::new(cwd))).or_default().push(id.clone());
        }
        self.closed.retain(|root, _| groups.contains_key(root));
        groups
            .into_iter()
            .map(|(root, sessions)| {
                let (closed, live): (Vec<_>, Vec<_>) =
                    read_tickets(&tickets_dir(&root)).into_iter().partition(|(_, t)| t.status == Status::Closed);
                let after = stale_days(&root) * DAY_MS;
                let stale = live
                    .iter()
                    .filter(|(p, t)| {
                        t.status == Status::Doing && modified_ms(p).is_some_and(|m| now.saturating_sub(m) >= after)
                    })
                    .map(|(_, t)| t.id.clone())
                    .collect();
                let tickets: Vec<Ticket> = live.into_iter().map(|(_, t)| t).collect();
                // Archive files are only listed, never parsed.
                let now_closed: HashSet<String> = closed
                    .into_iter()
                    .map(|(_, t)| t.id)
                    .chain(md_files(&archive_dir(&root)).iter().filter_map(|p| id_from_name(p)))
                    .collect();
                let (seen, last) = self.closed.entry(root.clone()).or_insert_with(|| (now_closed.clone(), None));
                if let Some(id) = now_closed.difference(seen).max() {
                    *last = Some(Closed { id: id.clone(), at: now });
                }
                *seen = now_closed;
                Repo { root: root.to_string_lossy().into_owned(), sessions, tickets, last_closed: last.clone(), stale }
            })
            .collect()
    }
}

pub fn spawn(store: Arc<Store>) {
    if let Err(e) = std::thread::Builder::new().name("tickets".into()).spawn(move || watch(&store)) {
        eprintln!("peepthis: tickets thread failed: {e}");
    }
}

/// Rereads every live repo on any change in its tickets or archive folder, and every `POLL` (new sessions, new folders).
fn watch(store: &Store) {
    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })
    .inspect_err(|e| eprintln!("peepthis: tickets watcher unavailable, polling only: {e}"))
    .ok();
    let mut watched: HashSet<PathBuf> = HashSet::new();
    let mut shelf = Shelf::default();
    loop {
        let repos = shelf.read(&store.live_cwds(), store::now_ms());
        if let Some(w) = watcher.as_mut() {
            let want: HashSet<PathBuf> = repos
                .iter()
                .flat_map(|r| [tickets_dir(Path::new(&r.root)), archive_dir(Path::new(&r.root))])
                .filter(|d| d.is_dir())
                .collect();
            let gone: Vec<PathBuf> = watched.difference(&want).cloned().collect();
            for d in gone {
                let _ = w.unwatch(&d);
                watched.remove(&d);
            }
            for d in want {
                if !watched.contains(&d) && w.watch(&d, RecursiveMode::NonRecursive).is_ok() {
                    watched.insert(d);
                }
            }
        }
        store.set_repos(repos, store::now_ms());
        match rx.recv_timeout(POLL) {
            Ok(first) => {
                std::thread::sleep(DEBOUNCE);
                let mut failed = first.is_err();
                for res in rx.try_iter() {
                    failed |= res.is_err();
                }
                // Rewatch everything next pass, as a folder deleted and recreated loses its OS watch.
                if let (true, Some(w)) = (failed, watcher.as_mut()) {
                    for d in watched.drain() {
                        let _ = w.unwatch(&d);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => std::thread::sleep(POLL),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("peepthis-tickets-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: PathBuf, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn s(id: &str, cwd: &Path) -> (String, String) {
        (id.into(), cwd.to_string_lossy().into_owned())
    }

    #[test]
    fn id_from_args_and_title_from_tickets_folder() {
        let dir = repo("default");
        write(dir.join("tickets/AV-12-thing.md"), "---\nid: AV-12\ntitle: \"Make the thing\"\n---\n\ntitle: not this\n");
        let cwd = dir.to_str();
        assert_eq!(link("close av-12", cwd), Some(TicketLink { id: "AV-12".into(), title: Some("Make the thing".into()) }));
        assert_eq!(link("AV-99", cwd), Some(TicketLink { id: "AV-99".into(), title: None }));
        assert_eq!(link("AV-1", cwd).unwrap().title, None, "AV-1 must not match AV-12");
        assert_eq!(link("34 extra-arg pre-x", cwd), None);
        assert_eq!(link("", cwd), None);
    }

    #[test]
    fn repo_setup_names_the_tickets_folder() {
        let dir = repo("setup");
        write(dir.join(".peep/setup.md"), "---\nprefix: W\ntickets: work/issues/\n---\n");
        write(dir.join("work/issues/W-3-x.md"), "---\ntitle: From setup\n---\n");
        write(dir.join("tickets/W-3-y.md"), "---\ntitle: Wrong folder\n---\n");
        assert_eq!(link("W-3", dir.to_str()).unwrap().title.as_deref(), Some("From setup"));
        assert_eq!(archive_dir(&dir), dir.join("work/issues/").join("archive"));
        write(dir.join(".peep/setup.md"), "---\ntickets: work/issues/\narchive: done/\n---\n");
        assert_eq!(archive_dir(&dir), dir.join("done/"));
    }

    #[test]
    fn shelf_parser_reads_fixture_folder_and_skips_malformed_index_and_archive() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tickets");
        let got: Vec<Ticket> = read_tickets(&dir).into_iter().map(|(_, t)| t).collect();
        let ids: Vec<(&str, Status)> = got.iter().map(|t| (t.id.as_str(), t.status)).collect();
        assert_eq!(
            ids,
            vec![
                ("AV-01", Status::Open),
                ("AV-02", Status::Doing),
                ("AV-03", Status::Blocked),
                ("AV-04", Status::Deferred),
                ("AV-05", Status::Done),
                ("AV-06", Status::Closed),
            ]
        );
        assert_eq!(got[0].title, "Open: with a colon");
        assert_eq!(got[1].title, "", "missing title is empty; id comes from the file name");
        assert_eq!(got[2].reason.as_deref(), Some("Needs the parser from AV-01"));
        assert_eq!(got[2].wake.as_deref(), Some("ticket:AV-01"));
        assert_eq!(got[3].wake.as_deref(), Some("date:2026-12-01"));
        assert_eq!(got[0].reason, None);
    }

    #[test]
    fn shelf_groups_sessions_by_git_root_and_skips_closed() {
        let a = repo("group-a");
        let b = repo("group-b");
        std::fs::create_dir_all(a.join(".git")).unwrap();
        write(a.join("tickets/AV-1-x.md"), "---\nid: AV-1\ntitle: X\nstatus: open\n---\n");
        write(a.join("tickets/AV-2-y.md"), "---\nid: AV-2\ntitle: Y\nstatus: closed\n---\n");
        std::fs::create_dir_all(a.join("src/deep")).unwrap();
        let repos = Shelf::default().read(&[s("s1", &a), s("s2", &a.join("src/deep")), s("s3", &b), ("s4".into(), String::new())], 1);
        assert_eq!(repos.len(), 2);
        let ra = repos.iter().find(|r| Path::new(&r.root) == a).unwrap();
        assert_eq!(ra.sessions, vec!["s1", "s2"]);
        assert_eq!(ra.tickets.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), vec!["AV-1"]);
        let rb = repos.iter().find(|r| Path::new(&r.root) == b).unwrap();
        assert!(rb.tickets.is_empty());
    }

    #[test]
    fn shelf_reports_a_ticket_closing_after_the_first_read() {
        let dir = repo("closing");
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        write(dir.join("tickets/AV-1-x.md"), "---\nid: AV-1\nstatus: done\n---\n");
        write(dir.join("tickets/AV-2-y.md"), "---\nid: AV-2\nstatus: open\n---\n");
        write(dir.join("tickets/archive/AV-0-old.md"), "---\nid: AV-0\nstatus: closed\n---\n");
        let sessions = [s("s1", &dir)];
        let mut shelf = Shelf::default();
        assert_eq!(shelf.read(&sessions, 1)[0].last_closed, None, "already-archived tickets are not news");

        std::fs::rename(dir.join("tickets/AV-1-x.md"), dir.join("tickets/archive/AV-1-x.md")).unwrap();
        let r = shelf.read(&sessions, 2);
        assert_eq!(r[0].last_closed, Some(Closed { id: "AV-1".into(), at: 2 }));
        assert_eq!(r[0].tickets.len(), 1);
        assert_eq!(shelf.read(&sessions, 3)[0].last_closed, Some(Closed { id: "AV-1".into(), at: 2 }));

        write(dir.join("tickets/AV-2-y.md"), "---\nid: AV-2\nstatus: closed\n---\n");
        assert_eq!(shelf.read(&sessions, 4)[0].last_closed, Some(Closed { id: "AV-2".into(), at: 4 }));
        std::fs::rename(dir.join("tickets/AV-2-y.md"), dir.join("tickets/archive/AV-2-y.md")).unwrap();
        assert_eq!(shelf.read(&sessions, 5)[0].last_closed.as_ref().unwrap().at, 4,"the archive move is the same closing");
    }

    #[test]
    fn doing_tickets_unchanged_for_stale_days_are_stale() {
        let dir = repo("stale");
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        write(dir.join("tickets/AV-1-x.md"), "---\nid: AV-1\nstatus: doing\n---\n");
        write(dir.join("tickets/AV-2-y.md"), "---\nid: AV-2\nstatus: open\n---\n");
        let changed = modified_ms(&dir.join("tickets/AV-1-x.md")).unwrap();
        let sessions = [s("s1", &dir)];
        let at = |now| Shelf::default().read(&sessions, now)[0].stale.clone();
        assert!(at(changed + STALE_DAYS_DEFAULT * DAY_MS - 1).is_empty());
        assert_eq!(at(changed + STALE_DAYS_DEFAULT * DAY_MS), vec!["AV-1"], "open tickets never go stale");
        write(dir.join(".peep/setup.md"), "---\nprefix: AV\nstale.days: 1\n---\n");
        assert_eq!(at(changed + DAY_MS), vec!["AV-1"], "the setup overrides the default");
        write(dir.join(".peep/setup.md"), "---\nprefix: AV\nstale.days: soon\n---\n");
        assert!(at(changed + DAY_MS).is_empty(), "an unreadable value falls back to the default");
    }
}
