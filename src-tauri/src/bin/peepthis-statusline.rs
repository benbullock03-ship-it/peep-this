//! Statusline wrapper (spec §6.3): runs the user's original statusline with the same stdin, then forwards the
//! payload to the widget. Every failure is silent; the original's output must always appear.
//! `hook <Event>` mode relays a hook payload instead (AV-73): prints the widget's reply body, nothing on failure, exit 0.

#[path = "../statusline_original.rs"]
#[allow(dead_code)]
mod statusline_original;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;

const TOKEN_HEADER: &str = "X-PeepThis-Token";
const CONNECT_TIMEOUT: Duration = Duration::from_millis(200);
const IO_TIMEOUT: Duration = Duration::from_millis(500);
/// Exit waits for the POST at most until this long after start; a slow original already covers it.
const POST_DEADLINE: Duration = Duration::from_millis(25);
const MAX_BODY: usize = 1024 * 1024;
/// Outlasts the broker's 60s hold and stays under the hook's 75s timeout (`hooks::PERMISSION_TIMEOUT_SECS`).
const PERMISSION_READ_TIMEOUT: Duration = Duration::from_secs(70);
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Shell {
    Posix(PathBuf),
    Cmd,
}

fn main() {
    let started = Instant::now();
    let mut payload = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut payload);
    let dir = statusline_original::dir();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let [mode, event] = args.as_slice() {
        if mode == "hook" {
            if let Some(reply) = relay_hook(&dir, event, &payload) {
                let mut out = std::io::stdout();
                let _ = out.write_all(&reply);
                let _ = out.flush();
            }
            std::process::exit(0);
        }
    }

    let (done_tx, done_rx) = mpsc::channel();
    let body = payload.clone();
    let post_dir = dir.clone();
    std::thread::spawn(move || {
        let _ = post(&post_dir, &body);
        let _ = done_tx.send(());
    });

    let code = std::fs::read(statusline_original::path(&dir))
        .ok()
        .and_then(|raw| statusline_original::command(&raw))
        .map(|cmd| run_original(&cmd, payload).unwrap_or(0))
        .unwrap_or(0);
    let _ = std::io::stdout().flush();
    let _ = done_rx.recv_timeout(POST_DEADLINE.saturating_sub(started.elapsed()));
    std::process::exit(code);
}

/// Runs `cmd` the way Claude Code runs a statusLine on Windows: through Git Bash, so `~` and `sh x.sh` work.
/// A command with no shell syntax skips the extra bash process (about 45ms on Git Bash).
fn run_original(cmd: &str, stdin: Vec<u8>) -> Option<i32> {
    let path: Vec<PathBuf> =
        std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    let home = std::env::var("HOME").ok().or_else(|| std::env::var("USERPROFILE").ok()).filter(|h| !h.is_empty());
    let direct = direct_argv(cmd, home.as_deref())
        .and_then(|argv| Some((find_program(&argv[0], &path, |p| p.is_file())?, argv)));
    let mut command = match direct {
        Some((exe, argv)) => {
            let mut c = Command::new(exe);
            c.args(&argv[1..]);
            c
        }
        None => shell_command(cmd, path),
    };
    command.stdin(Stdio::piped()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().ok()?;
    let mut pipe = child.stdin.take()?;
    // A separate writer, so a child that never reads stdin cannot deadlock on a full pipe.
    let writer = std::thread::spawn(move || {
        let _ = pipe.write_all(&stdin);
    });
    let status = child.wait().ok()?;
    let _ = writer.join();
    Some(status.code().unwrap_or(1))
}

/// Whitespace-split words with `~` expanded, or `None` if a shell is needed to get the same meaning.
fn direct_argv(cmd: &str, home: Option<&str>) -> Option<Vec<String>> {
    const META: &str = "|&;<>()$`\\\"'*?[]{}=!#%^\n\r";
    if cmd.chars().any(|c| META.contains(c)) {
        return None;
    }
    let argv: Vec<String> = cmd
        .split_whitespace()
        .map(|w| match w.strip_prefix('~') {
            Some(rest) if rest.is_empty() || rest.starts_with('/') => {
                home.map(|h| format!("{}{rest}", h.replace('\\', "/").trim_end_matches('/')))
            }
            Some(_) => None,
            None => Some(w.to_string()),
        })
        .collect::<Option<_>>()?;
    (!argv.is_empty()).then_some(argv)
}

/// A bare program name resolved on PATH (WSL stubs skipped); names with a path or extension go via the shell.
fn find_program(name: &str, path: &[PathBuf], exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    if name.contains(['/', ':', '.']) {
        return None;
    }
    path.iter().filter(|d| !is_wsl_stub_dir(d)).map(|d| d.join(format!("{name}.exe"))).find(|p| exists(p))
}

fn shell_command(cmd: &str, path: Vec<PathBuf>) -> Command {
    let shell = choose_shell(
        std::env::var_os("CLAUDE_CODE_GIT_BASH_PATH").map(PathBuf::from),
        path,
        |p| p.is_file(),
    );
    match &shell {
        Shell::Posix(exe) => {
            let mut c = Command::new(exe);
            c.arg("-c").arg(cmd);
            c
        }
        Shell::Cmd => {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(cmd);
            c
        }
    }
}

/// `CLAUDE_CODE_GIT_BASH_PATH`, else bash/sh on PATH (skipping the WSL stubs), else Git for Windows' bash
/// found next to `git.exe`, else `cmd /C`.
fn choose_shell(git_bash_env: Option<PathBuf>, path: Vec<PathBuf>, exists: impl Fn(&Path) -> bool) -> Shell {
    if let Some(p) = git_bash_env.filter(|p| exists(p)) {
        return Shell::Posix(p);
    }
    let usable: Vec<&PathBuf> = path.iter().filter(|d| !is_wsl_stub_dir(d)).collect();
    for name in ["bash.exe", "sh.exe"] {
        if let Some(p) = usable.iter().map(|d| d.join(name)).find(|p| exists(p)) {
            return Shell::Posix(p);
        }
    }
    for d in &usable {
        if !exists(&d.join("git.exe")) {
            continue;
        }
        // Git\cmd\git.exe and Git\mingw64\bin\git.exe both sit under the Git root that holds bin\bash.exe.
        for root in d.ancestors().skip(1).take(2) {
            let bash = root.join("bin").join("bash.exe");
            if exists(&bash) {
                return Shell::Posix(bash);
            }
        }
    }
    Shell::Cmd
}

fn is_wsl_stub_dir(d: &Path) -> bool {
    let s = d.to_string_lossy().replace('/', "\\").to_ascii_lowercase();
    let s = s.trim_end_matches('\\');
    s.ends_with("\\windows\\system32") || s.ends_with("\\microsoft\\windowsapps")
}

fn post(dir: &Path, body: &[u8]) -> Option<u16> {
    if body.is_empty() || body.len() > MAX_BODY {
        return None;
    }
    let raw = std::fs::read(dir.join("endpoint.json")).ok()?;
    let (port, token) = parse_endpoint(&raw)?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut s = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).ok()?;
    s.set_write_timeout(Some(IO_TIMEOUT)).ok()?;
    s.set_read_timeout(Some(IO_TIMEOUT)).ok()?;
    s.write_all(&build_request("/statusline", port, &token, body)).ok()?;
    let mut head = [0u8; 32];
    let n = s.read(&mut head).ok()?;
    status_code(&head[..n])
}

/// The reply body of a 2xx answer to `/hook/<event>`, or `None` on any failure.
fn relay_hook(dir: &Path, event: &str, body: &[u8]) -> Option<Vec<u8>> {
    if body.is_empty() || body.len() > MAX_BODY || event.is_empty() || !event.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let raw = std::fs::read(dir.join("endpoint.json")).ok()?;
    let (port, token) = parse_endpoint(&raw)?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut s = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).ok()?;
    let read = if event == "PermissionRequest" { PERMISSION_READ_TIMEOUT } else { IO_TIMEOUT };
    s.set_write_timeout(Some(IO_TIMEOUT)).ok()?;
    s.set_read_timeout(Some(read)).ok()?;
    s.write_all(&build_request(&format!("/hook/{event}"), port, &token, body)).ok()?;
    let mut resp = Vec::new();
    s.take(MAX_BODY as u64).read_to_end(&mut resp).ok()?;
    let split = resp.windows(4).position(|w| w == b"\r\n\r\n")?;
    status_code(&resp[..split]).filter(|c| (200..300).contains(c))?;
    Some(resp[split + 4..].to_vec())
}

fn parse_endpoint(raw: &[u8]) -> Option<(u16, String)> {
    let v: Value = serde_json::from_slice(raw).ok()?;
    let port = u16::try_from(v.get("port")?.as_u64()?).ok().filter(|p| *p != 0)?;
    let token = v.get("token")?.as_str().filter(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_alphanumeric()))?;
    Some((port, token.to_owned()))
}

fn build_request(path: &str, port: u16, token: &str, body: &[u8]) -> Vec<u8> {
    let mut req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\n{TOKEN_HEADER}: {token}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    req.extend_from_slice(body);
    req
}

fn status_code(head: &[u8]) -> Option<u16> {
    std::str::from_utf8(head).ok()?.split_whitespace().nth(1)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::net::TcpListener;

    fn fs(files: &[&str]) -> impl Fn(&Path) -> bool {
        let set: HashSet<PathBuf> = files.iter().map(PathBuf::from).collect();
        move |p: &Path| set.contains(p)
    }

    fn dirs(ds: &[&str]) -> Vec<PathBuf> {
        ds.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn shell_prefers_claude_code_git_bash_override() {
        let exists = fs(&[r"D:\Git\bin\bash.exe", r"C:\Program Files\Git\usr\bin\bash.exe"]);
        let shell = choose_shell(
            Some(PathBuf::from(r"D:\Git\bin\bash.exe")),
            dirs(&[r"C:\Program Files\Git\usr\bin"]),
            exists,
        );
        assert_eq!(shell, Shell::Posix(PathBuf::from(r"D:\Git\bin\bash.exe")));
    }

    #[test]
    fn shell_ignores_missing_override_and_wsl_stubs() {
        let exists = fs(&[
            r"C:\WINDOWS\system32\bash.exe",
            r"C:\Users\u\AppData\Local\Microsoft\WindowsApps\bash.exe",
            r"C:\Program Files\Git\usr\bin\sh.exe",
        ]);
        let shell = choose_shell(
            Some(PathBuf::from(r"D:\gone\bash.exe")),
            dirs(&[r"C:\WINDOWS\system32", r"C:\Users\u\AppData\Local\Microsoft\WindowsApps\", r"C:\Program Files\Git\usr\bin"]),
            exists,
        );
        assert_eq!(shell, Shell::Posix(PathBuf::from(r"C:\Program Files\Git\usr\bin\sh.exe")));
    }

    #[test]
    fn shell_finds_git_bash_from_git_on_path() {
        let exists = fs(&[r"C:\Program Files\Git\cmd\git.exe", r"C:\Program Files\Git\bin\bash.exe"]);
        assert_eq!(
            choose_shell(None, dirs(&[r"C:\Program Files\Git\cmd"]), &exists),
            Shell::Posix(PathBuf::from(r"C:\Program Files\Git\bin\bash.exe"))
        );
        let exists = fs(&[r"E:\Git\mingw64\bin\git.exe", r"E:\Git\bin\bash.exe"]);
        assert_eq!(
            choose_shell(None, dirs(&[r"E:\Git\mingw64\bin"]), exists),
            Shell::Posix(PathBuf::from(r"E:\Git\bin\bash.exe"))
        );
    }

    #[test]
    fn shell_falls_back_to_cmd() {
        let exists = fs(&[r"C:\WINDOWS\system32\bash.exe"]);
        assert_eq!(choose_shell(None, dirs(&[r"C:\WINDOWS\system32", r"C:\tools"]), exists), Shell::Cmd);
    }

    #[test]
    fn simple_commands_skip_the_shell_with_tilde_expanded() {
        let home = Some(r"C:\Users\u");
        assert_eq!(
            direct_argv("sh ~/.claude/statusline-command.sh", home),
            Some(vec!["sh".into(), "C:/Users/u/.claude/statusline-command.sh".into()])
        );
        assert_eq!(direct_argv("  node   C:/x/s.js  ", None), Some(vec!["node".into(), "C:/x/s.js".into()]));
        assert_eq!(direct_argv("ls ~", Some("C:/Users/u/")), Some(vec!["ls".into(), "C:/Users/u".into()]));
    }

    #[test]
    fn shell_syntax_needs_the_shell() {
        let home = Some("C:/Users/u");
        for cmd in [
            "",
            "   ",
            "sh a.sh | head",
            "echo $HOME",
            "a && b",
            "sh 'my script.sh'",
            r"node C:\x\s.js",
            "FOO=1 sh a.sh",
            "cat ~other/x",
            "ls *.sh",
            "echo `date`",
            "a\nb",
        ] {
            assert_eq!(direct_argv(cmd, home), None, "{cmd:?}");
        }
        assert_eq!(direct_argv("sh ~/a.sh", None), None);
    }

    #[test]
    fn program_lookup_skips_wsl_and_paths() {
        let path = dirs(&[r"C:\WINDOWS\system32", r"C:\Program Files\Git\usr\bin"]);
        let exists = fs(&[r"C:\WINDOWS\system32\bash.exe", r"C:\Program Files\Git\usr\bin\sh.exe"]);
        assert_eq!(find_program("sh", &path, &exists), Some(PathBuf::from(r"C:\Program Files\Git\usr\bin\sh.exe")));
        assert_eq!(find_program("bash", &path, &exists), None);
        assert_eq!(find_program("C:/tools/sh", &path, &exists), None);
        assert_eq!(find_program("sh.exe", &path, &exists), None);
    }

    #[test]
    fn endpoint_parse() {
        let tok = "ab".repeat(32);
        let raw = format!(r#"{{"port":47821,"token":"{tok}"}}"#);
        assert_eq!(parse_endpoint(raw.as_bytes()), Some((47821, tok)));
        for bad in [
            "{nope",
            r#"{"port":47821}"#,
            r#"{"token":"abc"}"#,
            r#"{"port":0,"token":"abc"}"#,
            r#"{"port":70000,"token":"abc"}"#,
            r#"{"port":1,"token":""}"#,
            r#"{"port":1,"token":"a\r\nX: y"}"#,
        ] {
            assert_eq!(parse_endpoint(bad.as_bytes()), None, "{bad}");
        }
    }

    #[test]
    fn request_bytes() {
        let req = build_request("/statusline", 47821, "tok", br#"{"a":1}"#);
        assert_eq!(
            String::from_utf8(req).unwrap(),
            "POST /statusline HTTP/1.1\r\nHost: 127.0.0.1:47821\r\nContent-Type: application/json\r\n\
             Content-Length: 7\r\nX-PeepThis-Token: tok\r\nConnection: close\r\n\r\n{\"a\":1}"
        );
    }

    #[test]
    fn status_line_parse() {
        assert_eq!(status_code(b"HTTP/1.1 204 No Content\r\n"), Some(204));
        assert_eq!(status_code(b""), None);
        assert_eq!(status_code(b"garbage"), None);
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("peepthis-statusline-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn posts_payload_to_endpoint() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let dir = temp_dir("post");
        std::fs::write(dir.join("endpoint.json"), format!(r#"{{"port":{port},"token":"t0k"}}"#)).unwrap();
        let server = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 4096];
            let mut got = Vec::new();
            while !got.ends_with(b"{\"session_id\":\"x\"}") {
                let n = s.read(&mut buf).unwrap();
                assert!(n > 0);
                got.extend_from_slice(&buf[..n]);
            }
            s.write_all(b"HTTP/1.1 204 No Content\r\n\r\n").unwrap();
            String::from_utf8(got).unwrap()
        });
        assert_eq!(post(&dir, br#"{"session_id":"x"}"#), Some(204));
        let got = server.join().unwrap();
        assert!(got.starts_with("POST /statusline HTTP/1.1\r\n"));
        assert!(got.contains("X-PeepThis-Token: t0k\r\n"));
        let _ = std::fs::remove_dir_all(dir);
    }

    fn one_reply_server(reply: impl AsRef<[u8]> + Send + 'static) -> (u16, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 4096];
            let mut got = Vec::new();
            while !got.ends_with(b"}") {
                let n = s.read(&mut buf).unwrap();
                assert!(n > 0);
                got.extend_from_slice(&buf[..n]);
            }
            s.write_all(reply.as_ref()).unwrap();
            String::from_utf8(got).unwrap()
        });
        (port, server)
    }

    #[test]
    fn hook_relay_passes_a_2xx_body_through() {
        let decision = r#"{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}"#;
        let reply = format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{decision}", decision.len());
        let (port, server) = one_reply_server(reply);
        let dir = temp_dir("hook200");
        std::fs::write(dir.join("endpoint.json"), format!(r#"{{"port":{port},"token":"t0k"}}"#)).unwrap();
        assert_eq!(relay_hook(&dir, "PermissionRequest", br#"{"session_id":"x"}"#), Some(decision.as_bytes().to_vec()));
        let got = server.join().unwrap();
        assert!(got.starts_with("POST /hook/PermissionRequest HTTP/1.1\r\n"), "{got}");
        assert!(got.contains("X-PeepThis-Token: t0k\r\n"));

        let (port, server) = one_reply_server(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n");
        std::fs::write(dir.join("endpoint.json"), format!(r#"{{"port":{port},"token":"t0k"}}"#)).unwrap();
        assert_eq!(relay_hook(&dir, "Stop", br#"{"session_id":"x"}"#), Some(Vec::new()));
        server.join().unwrap();

        let (port, server) = one_reply_server(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 2\r\n\r\nno");
        std::fs::write(dir.join("endpoint.json"), format!(r#"{{"port":{port},"token":"t0k"}}"#)).unwrap();
        assert_eq!(relay_hook(&dir, "Stop", br#"{"session_id":"x"}"#), None);
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn hook_relay_is_silent_without_endpoint_or_with_a_refused_port() {
        let dir = temp_dir("hooknone");
        assert_eq!(relay_hook(&dir, "Stop", b"{}"), None);
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        std::fs::write(dir.join("endpoint.json"), format!(r#"{{"port":{port},"token":"t"}}"#)).unwrap();
        assert_eq!(relay_hook(&dir, "Stop", b"{}"), None);
        assert_eq!(relay_hook(&dir, "../x", b"{}"), None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn post_is_silent_without_endpoint_or_payload() {
        let dir = temp_dir("none");
        assert_eq!(post(&dir, b"{}"), None);
        std::fs::write(dir.join("endpoint.json"), r#"{"port":1,"token":"t"}"#).unwrap();
        assert_eq!(post(&dir, b""), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
