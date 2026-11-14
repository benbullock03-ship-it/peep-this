//! AV-29: OS probe for Claude Code background shells (spec §11.7).
//!
//! A shell runs as `claude.exe -> bash.exe -c "... eval '<command>' ..."`, one direct child of the session
//! process per shell. Its output goes to `<tmp>/claude/<hash>/<session id>/tasks/<task id>.output`, which ends
//! with `[exited with code N]` or `[killed]` once the shell is done. Neither path is a documented contract.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::source::process;
use crate::source::ShellQuery;
use crate::state::{ShellExit, ShellObservation};

const TAIL_BYTES: u64 = 8 * 1024;
const LINE_MAX_CHARS: usize = 200;

pub fn probe(queries: &[ShellQuery]) -> Vec<ShellObservation> {
    let parents = process::parent_table();
    let ports = process::listening_ports();
    let os = Os {
        parents: &parents,
        ports: &ports,
        started: &process::started,
        command_line: &process::command_line,
    };
    probe_with(queries, &os, &tmp_root())
}

/// The OS facts a probe pass reads, injectable for tests.
pub struct Os<'a> {
    pub parents: &'a HashMap<u32, u32>,
    pub ports: &'a [(u32, u16)],
    pub started: &'a dyn Fn(u32) -> Option<u64>,
    pub command_line: &'a dyn Fn(u32) -> Option<String>,
}

pub fn probe_with(queries: &[ShellQuery], os: &Os, tmp: &Path) -> Vec<ShellObservation> {
    let pids = match_pids(queries, os);
    queries
        .iter()
        .zip(pids)
        .map(|(q, pid)| {
            let mut obs = ShellObservation::default();
            if let Some(file) = output_file(tmp, &q.session_id, &q.shell_id) {
                obs.last_output_at = modified_ms(&file);
                if let Some(tail) = read_tail(&file) {
                    let (line, exit) = parse_tail(&tail);
                    obs.last_line = line;
                    obs.exit = exit;
                }
            }
            if !q.ended {
                obs.pid = pid;
                obs.port = pid.and_then(|p| port_in_tree(p, os));
            }
            obs
        })
        .collect()
}

/// Each running query's wrapper PID: its earlier match if still a child of the session, else the earliest-started
/// unclaimed child whose command line contains the command.
fn match_pids(queries: &[ShellQuery], os: &Os) -> Vec<Option<u32>> {
    let mut children: HashMap<u32, Vec<(u32, Option<u64>)>> = HashMap::new();
    let mut out = vec![None; queries.len()];
    let mut claimed = HashSet::new();
    for q in queries.iter().filter(|q| !q.ended && q.session_pid != 0) {
        children.entry(q.session_pid).or_insert_with(|| {
            let ps = (os.started)(q.session_pid);
            let mut kids: Vec<_> = os
                .parents
                .iter()
                .filter(|(c, p)| **p == q.session_pid && **c != q.session_pid)
                .map(|(c, _)| (*c, (os.started)(*c)))
                .filter(|(_, cs)| !matches!((cs, ps), (Some(cs), Some(ps)) if *cs < ps))
                .collect();
            kids.sort_by_key(|(pid, st)| (*st, *pid));
            kids
        });
    }
    for (i, q) in queries.iter().enumerate() {
        if let Some(pid) = q.pid.filter(|p| !q.ended && children.get(&q.session_pid).is_some_and(|k| k.iter().any(|(c, _)| c == p))) {
            out[i] = Some(pid);
            claimed.insert(pid);
        }
    }
    let mut lines: HashMap<u32, Option<String>> = HashMap::new();
    for (i, q) in queries.iter().enumerate() {
        if out[i].is_some() || q.ended {
            continue;
        }
        let want = normalise(&q.command);
        if want.is_empty() {
            continue;
        }
        let Some(kids) = children.get(&q.session_pid) else { continue };
        out[i] = kids.iter().map(|(c, _)| *c).find(|c| {
            !claimed.contains(c)
                && lines
                    .entry(*c)
                    .or_insert_with(|| (os.command_line)(*c).map(|l| normalise(&l)))
                    .as_deref()
                    .is_some_and(|l| l.contains(&want))
        });
        if let Some(c) = out[i] {
            claimed.insert(c);
        }
    }
    out
}

/// Letters and digits only, lowercased: the wrapper re-quotes and escapes the command inside `eval '...'`.
pub fn normalise(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Lowest listening port held by the wrapper or any of its descendants.
fn port_in_tree(root: u32, os: &Os) -> Option<u16> {
    let mut tree = process::descendants(os.parents, root, os.started);
    tree.push(root);
    os.ports.iter().filter(|(pid, _)| tree.contains(pid)).map(|(_, port)| *port).min()
}

/// Claude Code's temp root: `CLAUDE_CODE_TMPDIR` or `%TEMP%`, plus `claude`.
fn tmp_root() -> PathBuf {
    std::env::var_os("CLAUDE_CODE_TMPDIR")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("claude")
}

pub fn output_file(tmp: &Path, session_id: &str, shell_id: &str) -> Option<PathBuf> {
    let safe = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !safe(session_id) || !safe(shell_id) {
        return None;
    }
    std::fs::read_dir(tmp)
        .ok()?
        .flatten()
        .map(|e| e.path().join(session_id).join("tasks").join(format!("{shell_id}.output")))
        .find(|p| p.is_file())
}

fn modified_ms(path: &Path) -> Option<u64> {
    let t = std::fs::metadata(path).ok()?.modified().ok()?;
    t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis() as u64)
}

fn read_tail(path: &Path) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    f.seek(SeekFrom::Start(len.saturating_sub(TAIL_BYTES))).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
}

/// Last non-blank output line and the exit trailer, if the shell has finished.
pub fn parse_tail(tail: &str) -> (Option<String>, Option<ShellExit>) {
    let mut lines = tail.lines().map(str::trim).filter(|l| !l.is_empty()).rev().peekable();
    let exit = match lines.peek().copied() {
        Some("[killed]") => Some(ShellExit::Killed),
        Some(l) => l
            .strip_prefix("[exited with code ")
            .and_then(|r| r.strip_suffix(']'))
            .and_then(|c| c.trim().parse().ok())
            .map(ShellExit::Code),
        None => None,
    };
    if exit.is_some() {
        lines.next();
    }
    let line = lines.next().map(|l| l.chars().take(LINE_MAX_CHARS).collect());
    (line, exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn q(session_pid: u32, id: &str, command: &str) -> ShellQuery {
        ShellQuery {
            session_id: "s1".into(),
            session_pid,
            shell_id: id.into(),
            command: command.into(),
            pid: None,
            ended: false,
        }
    }

    fn temp_dir() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!("peepthis-shells-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    // Captured wrapper command line (AV-29 run 2), anonymised.
    const WRAPPER: &str = r#""C:\Program Files\Git\bin\bash.exe" -c "source /c/Users/user/.claude/shell-snapshots/snapshot-bash-1-x.sh 2>/dev/null || true && eval 'node -e \"require(\\\"http\\\").createServer((q,r)=>r.end(\\\"hi\\\")).listen(47932,()=>console.log(\\\"listening 47932\\\"));setTimeout(()=>process.exit(0),45000)\"' < /dev/null && pwd -P >| /c/Users/user/AppData/Local/Temp/claude-b949-cwd""#;
    const COMMAND: &str = r#"node -e "require(\"http\").createServer((q,r)=>r.end(\"hi\")).listen(47932,()=>console.log(\"listening 47932\"));setTimeout(()=>process.exit(0),45000)""#;

    #[test]
    fn normalised_command_matches_the_escaped_wrapper_line() {
        assert!(normalise(WRAPPER).contains(&normalise(COMMAND)));
        assert!(!normalise(WRAPPER).contains(&normalise("sleep 60")));
    }

    #[test]
    fn tail_parsing_reads_trailers_and_last_lines() {
        assert_eq!(parse_tail("build broke\n\n[exited with code 3]\n"), (Some("build broke".into()), Some(ShellExit::Code(3))));
        assert_eq!(parse_tail("listening 47931\n\n[killed]\n"), (Some("listening 47931".into()), Some(ShellExit::Killed)));
        assert_eq!(parse_tail("\n[killed]\n"), (None, Some(ShellExit::Killed)));
        assert_eq!(parse_tail("ok\n\n[exited with code 0]\n"), (Some("ok".into()), Some(ShellExit::Code(0))));
        assert_eq!(parse_tail("compiling\r\nready in 300ms\r\n"), (Some("ready in 300ms".into()), None));
        assert_eq!(parse_tail(""), (None, None));
        assert_eq!(parse_tail(&"x".repeat(500)).0.unwrap().chars().count(), LINE_MAX_CHARS);
    }

    #[test]
    fn matches_children_by_command_in_start_order_and_finds_ports_in_the_tree() {
        // 100 = session; 201/202/203 = wrappers; 301 = node under 202; 900 = unrelated.
        let parents = HashMap::from([(201, 100), (202, 100), (203, 100), (301, 202), (900, 1)]);
        let ports = vec![(301, 47932), (301, 9229), (900, 80)];
        let lines = |pid: u32| match pid {
            201 => Some("bash -c \"eval 'sleep 60' \"".to_string()),
            202 => Some(WRAPPER.to_string()),
            203 => Some("bash -c \"eval 'sleep 60' \"".to_string()),
            _ => None,
        };
        let started = |pid: u32| Some(pid as u64);
        let os = Os { parents: &parents, ports: &ports, started: &started, command_line: &lines };
        let qs = vec![q(100, "srv", COMMAND), q(100, "a", "sleep 60"), q(100, "b", "sleep 60"), q(100, "c", "sleep 60"), q(0, "p", "x")];
        let obs = probe_with(&qs, &os, Path::new("Z:\\nope"));
        let got: Vec<_> = obs.iter().map(|o| (o.pid, o.port)).collect();
        assert_eq!(got, vec![(Some(202), Some(9229)), (Some(201), None), (Some(203), None), (None, None), (None, None)]);

        let mut known = q(100, "b", "sleep 60");
        known.pid = Some(203);
        let obs = probe_with(&[q(100, "a", "sleep 60"), known], &os, Path::new("Z:\\nope"));
        assert_eq!((obs[0].pid, obs[1].pid), (Some(201), Some(203)), "a known PID is kept and not reassigned");
    }

    #[test]
    fn output_file_gives_last_line_trailer_and_write_time() {
        let tmp = temp_dir();
        let tasks = tmp.join("b79b65889ee235ad").join("s1").join("tasks");
        std::fs::create_dir_all(&tasks).unwrap();
        std::fs::create_dir_all(tmp.join("other")).unwrap();
        std::fs::write(tasks.join("bx33ejiyl.output"), "build broke\n\n[exited with code 3]\n").unwrap();
        std::fs::write(tasks.join("b6s7b19v8.output"), "listening 47931\n").unwrap();
        let empty = HashMap::new();
        let os = Os { parents: &empty, ports: &[], started: &|_| None, command_line: &|_| None };
        let mut failed = q(0, "bx33ejiyl", "x");
        failed.ended = true;
        let obs = probe_with(&[failed, q(0, "b6s7b19v8", "y"), q(0, "../x", "z")], &os, &tmp);
        assert_eq!((obs[0].last_line.as_deref(), obs[0].exit), (Some("build broke"), Some(ShellExit::Code(3))));
        assert_eq!((obs[1].last_line.as_deref(), obs[1].exit), (Some("listening 47931"), None));
        let now = crate::store::now_ms();
        assert!(obs[1].last_output_at.is_some_and(|t| t <= now && now - t < 60_000));
        assert_eq!(obs[2], ShellObservation::default());
        let _ = std::fs::remove_dir_all(tmp);
    }

    #[cfg(windows)]
    #[test]
    fn real_probe_matches_a_child_shell_and_its_listening_port() {
        // This test process stands in for the session; a PowerShell child listens like a dev server would.
        let script = "$l=[System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback,0);$l.Start();Start-Sleep 20";
        let mut child = std::process::Command::new("powershell").args(["-NoProfile", "-Command", script]).spawn().unwrap();
        let me = std::process::id();
        let query = ShellQuery { session_pid: me, ..q(me, "b1", script) };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let obs = loop {
            let obs = probe(std::slice::from_ref(&query)).remove(0);
            if obs.port.is_some() || std::time::Instant::now() > deadline {
                break obs;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        };
        let _ = child.kill();
        let _ = child.wait();
        assert_eq!(obs.pid, Some(child.id()));
        assert!(obs.port.is_some_and(|p| p > 0), "{obs:?}");
    }
}
