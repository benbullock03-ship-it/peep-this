//! Loopback event server for hook and statusline payloads (spec §6.5).

use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;

use super::http;
use crate::approve::{self, Broker, TcpResponder};
use crate::state::{Event, ShellEvent, ShellExit};
use crate::store::{self, Store};

pub const DEFAULT_PORT: u16 = 47821;
pub const MAX_BODY: usize = 1024 * 1024;
pub const TOKEN_HEADER: &str = "X-PeepThis-Token";
const TEXT_MAX_CHARS: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub port: u16,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Hook(String),
    Statusline,
}

/// Everything kept from a payload; the rest is dropped at parse time.
#[derive(Debug, Clone, PartialEq)]
pub struct Routed {
    pub label: String,
    pub session_id: String,
    pub cwd: Option<String>,
    pub event: Option<Event>,
    pub rate_limits: Option<(f64, f64)>,
}

pub struct Request<'a> {
    pub post: bool,
    pub url: &'a str,
    pub token: Option<&'a str>,
    pub declared_len: Option<usize>,
}

/// `%APPDATA%\PeepThis`.
pub fn default_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("PeepThis")
}

/// What `handle` decided: answer now, or hand the connection to the approval broker.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Status(u16),
    Hold(approve::Request),
}

impl Outcome {
    /// The status sent when nothing holds the request.
    pub fn status(&self) -> u16 {
        match self {
            Outcome::Status(s) => *s,
            Outcome::Hold(_) => 204,
        }
    }
}

/// Binds 127.0.0.1:47821 (random port if taken), writes `endpoint.json` into `dir` and serves on a background thread.
pub fn spawn(store: Arc<Store>, broker: Option<Arc<Broker>>, dir: &Path) -> io::Result<Endpoint> {
    spawn_on(store, broker, dir, DEFAULT_PORT)
}

fn spawn_on(store: Arc<Store>, broker: Option<Arc<Broker>>, dir: &Path, port: u16) -> io::Result<Endpoint> {
    let listener = TcpListener::bind(("127.0.0.1", port)).or_else(|e| {
        println!("peepthis: port {port} unavailable ({e}), using a random port; installed hooks will not reach it");
        TcpListener::bind(("127.0.0.1", 0))
    })?;
    let port = listener.local_addr()?.port();
    // Native HTTP hooks carry the token literally in settings.json, so it must survive restarts.
    let token = match saved_token(dir) {
        Some(t) => t,
        None => new_token()?,
    };
    write_endpoint(dir, port, &token)?;
    println!("peepthis: event server on 127.0.0.1:{port}");

    let expected: Arc<str> = token.clone().into();
    std::thread::Builder::new()
        .name("event-server".into())
        .spawn(move || {
            for conn in listener.incoming().flatten() {
                let (store, broker, expected) = (Arc::clone(&store), broker.clone(), Arc::clone(&expected));
                // One thread per connection: a held approval must not block the hooks behind it.
                let _ = std::thread::Builder::new()
                    .name("event-conn".into())
                    .spawn(move || serve(&store, broker.as_deref(), &expected, conn));
            }
        })?;
    Ok(Endpoint { port, token })
}

fn serve(store: &Store, broker: Option<&Broker>, expected: &str, mut conn: TcpStream) {
    let _ = conn.set_read_timeout(Some(http::READ_TIMEOUT));
    let req = match http::read_request(&mut conn) {
        Ok(r) => r,
        Err(status) => {
            let _ = http::respond(&conn, status, None);
            return;
        }
    };
    let meta = Request {
        post: req.method == "POST",
        url: &req.url,
        token: req.header(TOKEN_HEADER),
        declared_len: req.declared_len,
    };
    let now = store::now_ms();
    match (handle_full(store, expected, &meta, &req.body[..], now), broker) {
        (Outcome::Hold(hold), Some(b)) => {
            let _ = conn.set_read_timeout(None);
            b.hold(hold, Box::new(TcpResponder(conn)), now);
        }
        (outcome, _) => {
            let _ = http::respond(&conn, outcome.status(), None);
        }
    }
}

/// Authorises, size-checks, parses and dispatches one request; returns the HTTP status.
#[cfg(test)]
pub fn handle(store: &Store, expected: &str, req: &Request, body: impl Read, now: u64) -> u16 {
    handle_full(store, expected, req, body, now).status()
}

pub fn handle_full(store: &Store, expected: &str, req: &Request, body: impl Read, now: u64) -> Outcome {
    if !token_matches(expected, req.token) {
        return Outcome::Status(401);
    }
    let Some(route) = parse_route(req.url) else { return Outcome::Status(404) };
    if !req.post {
        return Outcome::Status(405);
    }
    if req.declared_len.is_some_and(|n| n > MAX_BODY) {
        return Outcome::Status(413);
    }
    let mut buf = Vec::new();
    if body.take(MAX_BODY as u64 + 1).read_to_end(&mut buf).is_err() {
        return Outcome::Status(400);
    }
    if buf.len() > MAX_BODY {
        return Outcome::Status(413);
    }
    match parse(&route, &buf) {
        Ok(routed) => {
            dispatch(store, routed, now);
            let hold = (route == Route::Hook("PermissionRequest".into()))
                .then(|| serde_json::from_slice::<Value>(&buf).ok())
                .flatten()
                .and_then(|v| approve::Request::from_payload(&v));
            hold.map_or(Outcome::Status(204), Outcome::Hold)
        }
        Err(e) => {
            println!("peepthis: dropped {:?} payload: {e}", route);
            Outcome::Status(400)
        }
    }
}

fn dispatch(store: &Store, r: Routed, now: u64) {
    let note = if r.event.is_none() && r.rate_limits.is_none() { " (ignored)" } else { "" };
    println!("peepthis: {} session {}{note}", r.label, r.session_id);
    if let Some((five, seven)) = r.rate_limits {
        store.set_rate_limits(five, seven, now);
    }
    if let Some(ev) = r.event {
        store.apply_or_placeholder(&r.session_id, r.cwd.as_deref(), ev, now);
    }
}

pub fn token_matches(expected: &str, got: Option<&str>) -> bool {
    let Some(got) = got else { return false };
    let (a, b) = (expected.as_bytes(), got.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let diff = a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y));
    std::hint::black_box(diff) == 0
}

pub fn parse_route(url: &str) -> Option<Route> {
    let path = url.split('?').next().unwrap_or("");
    if path == "/statusline" {
        return Some(Route::Statusline);
    }
    let name = path.strip_prefix("/hook/")?;
    (!name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric()))
        .then(|| Route::Hook(name.to_string()))
}

pub fn parse(route: &Route, body: &[u8]) -> Result<Routed, &'static str> {
    let v: Value = serde_json::from_slice(body).map_err(|_| "malformed JSON")?;
    let session_id = str_at(&v, "/session_id")
        .filter(|s| !s.trim().is_empty())
        .ok_or("missing session_id")?;
    let cwd = str_at(&v, "/cwd").or_else(|| str_at(&v, "/workspace/current_dir"));
    let routed = match route {
        Route::Hook(name) => Routed {
            label: format!("hook {name}"),
            event: hook_event(name, &v, cwd.as_deref()).map(|e| {
                let e = scope_to_agent(e, &v);
                match shell_update(name, &v) {
                    Some(update) => Event::Shell { update, event: Box::new(e) },
                    None => e,
                }
            }),
            rate_limits: None,
            session_id,
            cwd,
        },
        Route::Statusline => {
            let model = str_at(&v, "/model/display_name");
            let context_pct = num_at(&v, "/context_window/remaining_percentage")
                .map(|r| (100.0 - r).clamp(0.0, 100.0))
                .or_else(|| num_at(&v, "/context_window/used_percentage"));
            let rate_limits = num_at(&v, "/rate_limits/five_hour/used_percentage")
                .zip(num_at(&v, "/rate_limits/seven_day/used_percentage"));
            Routed {
                label: "statusline".into(),
                event: Some(Event::Statusline { model, context_pct }),
                rate_limits,
                session_id,
                cwd,
            }
        }
    };
    Ok(routed)
}

fn hook_event(name: &str, v: &Value, cwd: Option<&str>) -> Option<Event> {
    Some(match name {
        "SessionStart" => Event::SessionStart,
        "UserPromptSubmit" => Event::UserPromptSubmit,
        "PreToolUse" => {
            let tool = str_at(v, "/tool_name")?;
            // The hand-back report is the subagent's result line (spec §11.6).
            let target = if tool == "SubagentHandback" {
                str_at(v, "/tool_input/message").and_then(|m| first_line(&m))
            } else {
                v.get("tool_input").and_then(|i| tool_target(i, cwd))
            };
            let description = match tool.as_str() {
                "Bash" | "PowerShell" => str_at(v, "/tool_input/description").and_then(|d| first_line(&d)),
                _ => None,
            };
            Event::PreToolUse { tool, target, description }
        }
        "PostToolUse" | "PostToolUseFailure" => Event::PostToolUse,
        // Fires after PreToolUse while the prompt is on screen; approval is followed by PostToolUse.
        "PermissionRequest" => {
            let tool = str_at(v, "/tool_name").unwrap_or_default();
            let target = v.get("tool_input").and_then(|i| tool_target(i, cwd));
            let what = match target {
                Some(t) => format!("{tool} {t}"),
                None => tool,
            };
            Event::Notification { message: clip(&format!("Needs permission: {what}")), idle: false }
        }
        "Notification" => {
            let message = str_at(v, "/message").unwrap_or_default();
            let idle = match str_at(v, "/notification_type") {
                Some(t) => t == "idle_prompt",
                None => message.contains("waiting for your input"),
            };
            Event::Notification { message: clip(&message), idle }
        }
        "Stop" => Event::Stop,
        "StopFailure" => Event::StopFailure { summary: failure_summary(v) },
        "SubagentStart" => Event::SubagentStart {
            agent_id: agent_id(v)?,
            agent_type: str_at(v, "/agent_type").unwrap_or_default(),
        },
        "SubagentStop" => Event::SubagentStop {
            agent_id: agent_id(v),
            result: str_at(v, "/last_assistant_message").and_then(|m| first_line(&m)),
        },
        "SessionEnd" => Event::Ended,
        _ => return None,
    })
}

fn agent_id(v: &Value) -> Option<String> {
    str_at(v, "/agent_id").filter(|s| !s.trim().is_empty())
}

/// Tool and notification events from inside a subagent carry its `agent_id` under the parent's `session_id`.
fn scope_to_agent(event: Event, v: &Value) -> Event {
    let Some(agent_id) = agent_id(v) else { return event };
    match event {
        Event::PreToolUse { .. } | Event::PostToolUse | Event::Notification { .. } => Event::Subagent {
            agent_id,
            agent_type: str_at(v, "/agent_type").unwrap_or_default(),
            event: Box::new(event),
        },
        other => other,
    }
}

/// Background shell lifecycle carried by a hook payload (spec §11.7).
fn shell_update(name: &str, v: &Value) -> Option<ShellEvent> {
    match name {
        "PostToolUse" => match str_at(v, "/tool_name")?.as_str() {
            "Bash" | "PowerShell" => Some(ShellEvent::Started {
                id: str_at(v, "/tool_response/backgroundTaskId").filter(|s| !s.trim().is_empty())?,
                command: clip(&str_at(v, "/tool_input/command").unwrap_or_default()),
                description: clip(&str_at(v, "/tool_input/description").unwrap_or_default()),
            }),
            "TaskStop" | "KillShell" => Some(ShellEvent::Ended {
                id: str_at(v, "/tool_input/task_id").or_else(|| str_at(v, "/tool_input/shell_id"))?,
                exit: ShellExit::Killed,
            }),
            _ => None,
        },
        "UserPromptSubmit" => task_notification(&str_at(v, "/prompt")?),
        "Stop" => {
            let shells: Vec<_> = v
                .get("background_tasks")?
                .as_array()?
                .iter()
                .filter(|t| t.get("type").and_then(Value::as_str) == Some("shell"))
                .filter_map(|t| {
                    let id = str_at(t, "/id").filter(|s| !s.trim().is_empty())?;
                    let command = str_at(t, "/command").map(|c| clip(&c)).unwrap_or_default();
                    let description = str_at(t, "/description").map(|d| clip(&d)).unwrap_or_default();
                    Some((id, command, description))
                })
                .collect();
            (!shells.is_empty()).then_some(ShellEvent::Listed { shells })
        }
        _ => None,
    }
}

/// A finished background task arrives as a `<task-notification>` prompt with its id, status and an exit-code summary.
fn task_notification(prompt: &str) -> Option<ShellEvent> {
    let body = prompt.trim_start().strip_prefix("<task-notification>")?;
    let tag = |t: &str| {
        let start = body.find(&format!("<{t}>"))? + t.len() + 2;
        let end = body[start..].find(&format!("</{t}>"))? + start;
        Some(body[start..end].trim().to_string())
    };
    let id = tag("task-id").filter(|s| !s.is_empty())?;
    let code = tag("summary").and_then(|s| exit_code_in(&s));
    let exit = match (tag("status").as_deref(), code) {
        (_, Some(c)) => ShellExit::Code(c),
        (Some("failed"), None) => ShellExit::Failed,
        (Some("killed" | "stopped"), None) => ShellExit::Killed,
        _ => ShellExit::Unknown,
    };
    Some(ShellEvent::Ended { id, exit })
}

/// `... failed with exit code 3` / `... completed (exit code 0)`.
fn exit_code_in(summary: &str) -> Option<i32> {
    let rest = &summary[summary.rfind("exit code")? + "exit code".len()..];
    let rest = rest.trim_start();
    let end = rest
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || (i == 0 && c == '-')))
        .map_or(rest.len(), |(i, _)| i);
    rest[..end].parse().ok()
}

/// First non-blank line, at most `TEXT_MAX_CHARS`.
fn first_line(s: &str) -> Option<String> {
    s.lines().map(str::trim).find(|l| !l.is_empty()).map(clip)
}

/// First useful string of the tool input; paths under `cwd` are made relative.
fn tool_target(input: &Value, cwd: Option<&str>) -> Option<String> {
    const KEYS: [&str; 9] = [
        "file_path",
        "notebook_path",
        "command",
        "pattern",
        "path",
        "url",
        "query",
        "description",
        "subagent_type",
    ];
    let (key, val) = KEYS.iter().find_map(|k| {
        input.get(*k).and_then(Value::as_str).filter(|s| !s.trim().is_empty()).map(|s| (*k, s))
    })?;
    let val = match (key, cwd) {
        ("file_path" | "notebook_path" | "path", Some(cwd)) if !cwd.is_empty() => val
            .strip_prefix(cwd.trim_end_matches(['/', '\\']))
            .and_then(|rest| rest.strip_prefix(['/', '\\']))
            .filter(|rest| !rest.is_empty())
            .unwrap_or(val),
        _ => val,
    };
    Some(clip(val))
}

/// `error` is a type code (`rate_limit`, `model_not_found`); `error_details` is optional prose like "429 Too Many Requests".
fn failure_summary(v: &Value) -> String {
    str_at(v, "/error_details")
        .filter(|s| !s.trim().is_empty())
        .or_else(|| str_at(v, "/error").map(|e| e.replace('_', " ")))
        .map(|d| clip(&format!("API error: {d}")))
        .unwrap_or_default()
}

/// First line, at most `TEXT_MAX_CHARS`; the state engine truncates further for display.
fn clip(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().chars().take(TEXT_MAX_CHARS).collect()
}

fn str_at(v: &Value, ptr: &str) -> Option<String> {
    v.pointer(ptr).and_then(Value::as_str).map(str::to_owned)
}

fn num_at(v: &Value, ptr: &str) -> Option<f64> {
    v.pointer(ptr).and_then(Value::as_f64)
}

fn new_token() -> io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| io::Error::other(e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn saved_token(dir: &Path) -> Option<String> {
    let raw = std::fs::read(dir.join("endpoint.json")).ok()?;
    let v: Value = serde_json::from_slice(&raw).ok()?;
    str_at(&v, "/token").filter(|t| t.len() == 64 && t.chars().all(|c| c.is_ascii_hexdigit()))
}

fn write_endpoint(dir: &Path, port: u16, token: &str) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join("endpoint.json.tmp");
    std::fs::write(&tmp, serde_json::json!({ "port": port, "token": token }).to_string())?;
    std::fs::rename(&tmp, dir.join("endpoint.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicU32, Ordering};

    const SID: &str = "aaaaaaaa-1111-4222-8333-444444444444";
    const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn fixture(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hooks").join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn hook(name: &str, file: &str) -> Routed {
        parse(&Route::Hook(name.into()), fixture(file).as_bytes()).unwrap()
    }

    fn quiet_store() -> Arc<Store> {
        Store::new(Box::new(|_| {}))
    }

    fn temp_dir() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "peepthis-ingest-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn req<'a>(url: &'a str, token: Option<&'a str>) -> Request<'a> {
        Request { post: true, url, token, declared_len: None }
    }

    fn send(store: &Store, r: &Request, body: &str) -> u16 {
        handle(store, TOKEN, r, body.as_bytes(), 1_000)
    }

    #[test]
    fn token_compare() {
        assert!(token_matches(TOKEN, Some(TOKEN)));
        assert!(!token_matches(TOKEN, None));
        assert!(!token_matches(TOKEN, Some("")));
        assert!(!token_matches(TOKEN, Some(&TOKEN[1..])));
        let mut wrong = TOKEN.to_string();
        wrong.replace_range(63.., "0");
        assert!(!token_matches(TOKEN, Some(&wrong)));
    }

    #[test]
    fn tokens_are_random_64_hex_chars() {
        let (a, b) = (new_token().unwrap(), new_token().unwrap());
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn routes() {
        assert_eq!(parse_route("/statusline"), Some(Route::Statusline));
        assert_eq!(parse_route("/hook/PreToolUse"), Some(Route::Hook("PreToolUse".into())));
        assert_eq!(parse_route("/hook/Stop?x=1"), Some(Route::Hook("Stop".into())));
        for bad in ["/", "/hook/", "/hook/../x", "/hook/a/b", "/statusline/x", "/hooks/Stop"] {
            assert_eq!(parse_route(bad), None, "{bad}");
        }
    }

    #[test]
    fn maps_every_hook_fixture_to_its_event() {
        let cases = [
            ("SessionStart", "session_start.json", Event::SessionStart),
            ("UserPromptSubmit", "user_prompt_submit.json", Event::UserPromptSubmit),
            (
                "PreToolUse",
                "pre_tool_use_edit.json",
                Event::PreToolUse { tool: "Edit".into(), target: Some("src\\main.ts".into()), description: None },
            ),
            (
                "PreToolUse",
                "pre_tool_use_bash.json",
                Event::PreToolUse {
                    tool: "Bash".into(),
                    target: Some("npm test".into()),
                    description: Some("Run the test suite".into()),
                },
            ),
            (
                "PreToolUse",
                "pre_tool_use_agent.json",
                Event::PreToolUse {
                    tool: "Agent".into(),
                    target: Some("Read note.txt and report contents".into()),
                    description: None,
                },
            ),
            ("PostToolUse", "post_tool_use.json", Event::PostToolUse),
            ("PostToolUseFailure", "post_tool_use_failure.json", Event::PostToolUse),
            (
                "PermissionRequest",
                "permission_request.json",
                Event::Notification { message: "Needs permission: Write perm.txt".into(), idle: false },
            ),
            (
                "Notification",
                "notification_permission.json",
                Event::Notification { message: "Claude needs your permission".into(), idle: false },
            ),
            (
                "Notification",
                "notification_idle.json",
                Event::Notification { message: "Claude is waiting for your input".into(), idle: true },
            ),
            ("Stop", "stop.json", Event::Stop),
            (
                "StopFailure",
                "stop_failure.json",
                Event::StopFailure { summary: "API error: model not found".into() },
            ),
            (
                "SubagentStop",
                "subagent_stop.json",
                Event::SubagentStop {
                    agent_id: Some("a0123456789abcdef".into()),
                    result: Some("note.txt says hello".into()),
                },
            ),
            (
                "SubagentStart",
                "subagent_start.json",
                Event::SubagentStart { agent_id: "a0123456789abcdef".into(), agent_type: "general-purpose".into() },
            ),
            (
                "PreToolUse",
                "subagent_pre_tool_use_read.json",
                Event::Subagent {
                    agent_id: "a665acd7e219f0015".into(),
                    agent_type: "general-purpose".into(),
                    event: Box::new(Event::PreToolUse { tool: "Read".into(), target: Some("b.txt".into()), description: None }),
                },
            ),
            (
                "PostToolUse",
                "subagent_post_tool_use_read.json",
                Event::Subagent {
                    agent_id: "a665acd7e219f0015".into(),
                    agent_type: "general-purpose".into(),
                    event: Box::new(Event::PostToolUse),
                },
            ),
            (
                "PermissionRequest",
                "permission_request_subagent.json",
                Event::Subagent {
                    agent_id: "a0123456789abcdef".into(),
                    agent_type: "general-purpose".into(),
                    event: Box::new(Event::Notification { message: "Needs permission: Write f.txt".into(), idle: false }),
                },
            ),
            ("PostToolUse", "post_tool_use_agent_async.json", Event::PostToolUse),
            ("SessionEnd", "session_end.json", Event::Ended),
            (
                "PostToolUse",
                "post_tool_use_bash_background.json",
                Event::Shell {
                    update: ShellEvent::Started {
                        id: "bx33ejiyl".into(),
                        command: "sleep 2; echo build broke >&2; exit 3".into(),
                        description: "Run failing command in background".into(),
                    },
                    event: Box::new(Event::PostToolUse),
                },
            ),
            (
                "UserPromptSubmit",
                "user_prompt_submit_task_notification.json",
                Event::Shell {
                    update: ShellEvent::Ended { id: "bx33ejiyl".into(), exit: ShellExit::Code(3) },
                    event: Box::new(Event::UserPromptSubmit),
                },
            ),
            (
                "PostToolUse",
                "post_tool_use_task_stop.json",
                Event::Shell {
                    update: ShellEvent::Ended { id: "bydeacx8e".into(), exit: ShellExit::Killed },
                    event: Box::new(Event::PostToolUse),
                },
            ),
            (
                "Stop",
                "stop_background_shell.json",
                Event::Shell {
                    update: ShellEvent::Listed {
                        shells: vec![(
                            "bv4qmnt44".into(),
                            r#"node -e "require(\"http\").createServer((q,r)=>r.end(\"hi\")).listen(47932,()=>console.log(\"listening 47932\"));setTimeout(()=>process.exit(0),45000)""#.into(),
                            "Start background HTTP server on port 47932".into(),
                        )],
                    },
                    event: Box::new(Event::Stop),
                },
            ),
        ];
        for (name, file, want) in cases {
            let r = hook(name, file);
            assert_eq!(r.session_id, SID);
            assert_eq!(r.cwd.as_deref(), Some("C:\\work\\demo"));
            assert_eq!(r.event, Some(want), "{file}");
            assert!(!format!("{r:?}").contains("SECRET"), "{file} leaked payload content");
        }
    }

    #[test]
    fn unknown_hook_names_parse_to_no_event() {
        assert_eq!(hook("PreCompact", "stop.json").event, None);
    }

    #[test]
    fn subagent_handback_message_is_the_target_and_events_without_agent_id_stay_unscoped() {
        let body = format!(
            r#"{{"session_id":"{SID}","agent_id":"a1","agent_type":"Explore","tool_name":"SubagentHandback","tool_input":{{"message":"\n Found 3 issues\nSECRET"}}}}"#
        );
        let r = parse(&Route::Hook("PreToolUse".into()), body.as_bytes()).unwrap();
        let Some(Event::Subagent { event, .. }) = r.event else { panic!("{r:?}") };
        assert_eq!(*event, Event::PreToolUse { tool: "SubagentHandback".into(), target: Some("Found 3 issues".into()), description: None });

        let internal = format!(r#"{{"session_id":"{SID}","agent_type":"","last_assistant_message":""}}"#);
        let r = parse(&Route::Hook("SubagentStop".into()), internal.as_bytes()).unwrap();
        assert_eq!(r.event, Some(Event::SubagentStop { agent_id: None, result: None }));
        let r = parse(&Route::Hook("SubagentStart".into()), internal.as_bytes()).unwrap();
        assert_eq!(r.event, None, "SubagentStart without agent_id is dropped");
        let r = parse(&Route::Hook("Stop".into()), fixture("subagent_stop.json").replace("SubagentStop", "Stop").as_bytes()).unwrap();
        assert_eq!(r.event, Some(Event::Stop), "only tool and notification events are scoped");
    }

    /// Replays a captured session through the HTTP handler; returns each request's status.
    fn replay(store: &Store, file: &str) -> Vec<u16> {
        fixture(file)
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let v: Value = serde_json::from_str(line).unwrap();
                let url = format!("/hook/{}", v["hook"].as_str().unwrap());
                let body = v["body"].to_string();
                let status = handle(store, TOKEN, &req(&url, Some(TOKEN)), body.as_bytes(), 1_000 + i as u64 * 100);
                let s = &store.snapshot().sessions[0];
                let agents: Vec<String> = s
                    .agents
                    .iter()
                    .map(|a| format!("{}={:?} {:?} tools={} result={:?}", &a.id[..4], a.state, a.task, a.tool_count, a.result))
                    .collect();
                println!("{url}: parent={:?} {:?} | {}", s.state, s.task, agents.join(" | "));
                status
            })
            .collect()
    }

    #[test]
    fn replayed_three_agent_fanout_gives_three_children_with_states_tools_and_results() {
        let store = quiet_store();
        let statuses = replay(&store, "subagent_fanout.jsonl");
        assert!(statuses.iter().all(|&c| c == 204), "{statuses:?}");
        let s = store.snapshot().sessions[0].clone();
        assert_eq!(s.state, State::Success);
        assert_eq!(s.task, "Done");
        assert_eq!(s.last_tool.as_ref().map(|t| t.name.as_str()), Some("Agent"), "subagent Reads never became the parent's tool");
        let got: Vec<_> = s
            .agents
            .iter()
            .map(|a| (a.description.as_str(), a.state, a.tool_count, a.result.as_deref(), a.ended_at.is_some()))
            .collect();
        assert_eq!(
            got,
            vec![
                ("Read a.txt first line", State::Success, 1, Some("apple"), true),
                ("Read b.txt first line", State::Success, 1, Some("banana"), true),
                ("Read c.txt first line", State::Success, 1, Some("cherry"), true),
            ]
        );
        assert!(s.agents.iter().all(|a| a.agent_type == "general-purpose"));
        assert_eq!(s.agents[1].last_tool, Some(crate::state::LastTool { name: "Read".into(), target: Some("b.txt".into()), description: None }));
        assert!(!format!("{s:?}").contains("SECRET"));
    }

    #[test]
    fn replayed_fanout_midway_shows_each_child_in_its_own_state() {
        let store = quiet_store();
        let lines: Vec<_> = fixture("subagent_fanout.jsonl").lines().take(8).map(str::to_owned).collect();
        for (i, line) in lines.iter().enumerate() {
            let v: Value = serde_json::from_str(line).unwrap();
            let url = format!("/hook/{}", v["hook"].as_str().unwrap());
            handle(&store, TOKEN, &req(&url, Some(TOKEN)), v["body"].to_string().as_bytes(), 1_000 + i as u64);
        }
        let s = store.snapshot().sessions[0].clone();
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Agent Read c.txt first line");
        let states: Vec<_> = s.agents.iter().map(|a| (a.state, a.task.as_str())).collect();
        assert_eq!(
            states,
            vec![(State::Thinking, "Thinking"), (State::Working, "Read b.txt"), (State::Thinking, "Thinking")]
        );
    }

    #[test]
    fn replayed_background_agent_outlives_the_turn() {
        let store = quiet_store();
        let statuses = replay(&store, "subagent_background.jsonl");
        assert!(statuses.iter().all(|&c| c == 204));
        let s = store.snapshot().sessions[0].clone();
        assert!(!s.alive);
        assert_eq!(s.agents.len(), 1);
        let a = &s.agents[0];
        assert_eq!(a.description, "Nested agent test with b.txt");
        assert_eq!((a.state, a.tool_count), (State::Success, 1));
        assert_eq!(a.result.as_deref(), Some("The first line of b.txt is `banana`."));
        assert!(!format!("{s:?}").contains("SECRET"));

        let store = quiet_store();
        let first: Vec<_> = fixture("subagent_background.jsonl").lines().take(6).map(str::to_owned).collect();
        for (i, line) in first.iter().enumerate() {
            let v: Value = serde_json::from_str(line).unwrap();
            let url = format!("/hook/{}", v["hook"].as_str().unwrap());
            handle(&store, TOKEN, &req(&url, Some(TOKEN)), v["body"].to_string().as_bytes(), 1_000 + i as u64);
        }
        let s = store.snapshot().sessions[0].clone();
        assert_eq!(s.state, State::Success, "parent's Stop holds while the background agent works");
        assert_eq!((s.agents[0].state, s.agents[0].task.as_str()), (State::Working, "Read b.txt"));
    }

    #[test]
    fn task_notifications_map_status_and_exit_code() {
        let note = |status: &str, summary: &str| {
            let prompt = format!("<task-notification>\n<task-id>b1</task-id>\n<status>{status}</status>\n<summary>{summary}</summary>\n</task-notification>");
            let body = serde_json::json!({ "session_id": SID, "prompt": prompt }).to_string();
            match parse(&Route::Hook("UserPromptSubmit".into()), body.as_bytes()).unwrap().event {
                Some(Event::Shell { update: ShellEvent::Ended { id, exit }, event }) if *event == Event::UserPromptSubmit => {
                    assert_eq!(id, "b1");
                    Some(exit)
                }
                Some(Event::UserPromptSubmit) => None,
                other => panic!("{other:?}"),
            }
        };
        assert_eq!(note("completed", "Background command \"x\" completed (exit code 0)"), Some(ShellExit::Code(0)));
        assert_eq!(note("failed", "Background command \"x\" failed with exit code 127"), Some(ShellExit::Code(127)));
        assert_eq!(note("failed", "Background command \"x\" failed"), Some(ShellExit::Failed));
        assert_eq!(note("killed", "Background command \"x\" was stopped"), Some(ShellExit::Killed));
        assert_eq!(note("completed", "Agent \"x\" completed"), Some(ShellExit::Unknown));
        assert_eq!(exit_code_in("failed with exit code -1"), Some(-1));

        let plain = format!(r#"{{"session_id":"{SID}","prompt":"run <task-notification> please"}}"#);
        let r = parse(&Route::Hook("UserPromptSubmit".into()), plain.as_bytes()).unwrap();
        assert_eq!(r.event, Some(Event::UserPromptSubmit));
    }

    #[test]
    fn foreground_bash_and_shell_less_stop_carry_no_shell_update() {
        assert_eq!(hook("PostToolUse", "post_tool_use.json").event, Some(Event::PostToolUse));
        assert_eq!(hook("Stop", "stop.json").event, Some(Event::Stop));
        let fg = format!(r#"{{"session_id":"{SID}","tool_name":"Bash","tool_input":{{"command":"ls"}},"tool_response":{{"stdout":"x"}}}}"#);
        assert_eq!(parse(&Route::Hook("PostToolUse".into()), fg.as_bytes()).unwrap().event, Some(Event::PostToolUse));
        let sub = format!(
            r#"{{"session_id":"{SID}","agent_id":"a1","agent_type":"Explore","tool_name":"PowerShell","tool_input":{{"command":"npm run dev"}},"tool_response":{{"backgroundTaskId":"b9"}}}}"#
        );
        let Some(Event::Shell { update, event }) = parse(&Route::Hook("PostToolUse".into()), sub.as_bytes()).unwrap().event else { panic!() };
        assert_eq!(update, ShellEvent::Started { id: "b9".into(), command: "npm run dev".into(), description: String::new() });
        assert!(matches!(*event, Event::Subagent { .. }), "a subagent's shell still belongs to the session");
    }

    fn send_line(store: &Store, line: &str, now: u64) -> u16 {
        let v: Value = serde_json::from_str(line).unwrap();
        let url = format!("/hook/{}", v["hook"].as_str().unwrap());
        handle(store, TOKEN, &req(&url, Some(TOKEN)), v["body"].to_string().as_bytes(), now)
    }

    fn shell_line(s: &crate::state::Session) -> String {
        s.shells
            .iter()
            .map(|sh| format!("{}={:?} port={:?} exit={:?} last={:?}", sh.id, sh.state, sh.port, sh.exit_code, sh.last_line))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// Replays the captured dev-server + failing-build run, with the ticker's probe pass after every request
    /// running the real probe logic against a fake process table and temp output files.
    #[test]
    fn replayed_dev_server_and_failing_build_show_serving_port_and_failed_exit_code() {
        use crate::source::claude::shells::{self, Os};
        use crate::source::{SessionInfo, SourceEvent};
        use crate::state::ShellState;
        use crate::ticker::{self, ShellProbe};
        use std::collections::HashMap;

        let lines: Vec<String> = fixture("shell_background.jsonl").lines().map(str::to_owned).collect();
        let server_cmd = serde_json::from_str::<Value>(&lines[1]).unwrap()["body"]["tool_input"]["command"].as_str().unwrap().to_string();
        let wrapper = format!(
            "\"C:\\Program Files\\Git\\bin\\bash.exe\" -c \"source snapshot.sh && eval '{}' < /dev/null && pwd -P\"",
            server_cmd.replace('"', "\\\"")
        );
        let tmp = temp_dir();
        let tasks = tmp.join("0123456789abcdef").join(SID).join("tasks");
        std::fs::create_dir_all(&tasks).unwrap();
        std::fs::write(tasks.join("b6s7b19v8.output"), "listening 47931\n").unwrap();

        let probe_tmp = tmp.clone();
        let probe: ShellProbe = Box::new(move |qs| {
            // 100 = claude.exe, 201 = the shell's bash wrapper, 301 = node holding the port, 202 = a foreground shell.
            let parents = HashMap::from([(201, 100), (202, 100), (301, 201)]);
            let ports = [(301u32, 47931u16)];
            let w = wrapper.clone();
            let lines = move |pid: u32| match pid {
                201 => Some(w.clone()),
                202 => Some("bash.exe -c \"eval 'sleep 5'\"".to_string()),
                _ => None,
            };
            let started = |pid: u32| Some(pid as u64);
            shells::probe_with(qs, &Os { parents: &parents, ports: &ports, started: &started, command_line: &lines }, &probe_tmp)
        });

        let store = quiet_store();
        let info = SessionInfo { id: SID.into(), pid: 100, name: "demo".into(), cwd: "C:\\work\\demo".into(), kind: "interactive".into(), entrypoint: "cli".into() };
        store.ingest(SourceEvent::Upsert { info, busy: None }, 1_000);
        let mut at_curl = None;
        for (i, line) in lines.iter().enumerate() {
            let now = 1_000 + i as u64 * 1_000;
            assert_eq!(send_line(&store, line, now), 204, "line {i}");
            if i == 7 {
                // Written after the notification so the exit comes from the hook and only the last line from the file.
                std::fs::write(tasks.join("bx33ejiyl.output"), "build broke\n\n[exited with code 3]\n").unwrap();
            }
            ticker::probe_shells(&store, &probe, now + 500);
            let s = store.snapshot().sessions[0].clone();
            println!("{}: {:?} | {}", line.split("\"hook\":\"").nth(1).unwrap().split('"').next().unwrap(), s.state, shell_line(&s));
            if i == 13 {
                at_curl = Some(s);
            }
        }

        let s = at_curl.unwrap();
        let (srv, build) = (&s.shells[0], &s.shells[1]);
        assert_eq!((srv.id.as_str(), srv.state, srv.port, srv.pid), ("b6s7b19v8", ShellState::Serving, Some(47931), Some(201)));
        assert_eq!(srv.last_line.as_deref(), Some("listening 47931"));
        assert_eq!((build.id.as_str(), build.state, build.exit_code), ("bx33ejiyl", ShellState::Failed, Some(3)));
        assert_eq!(build.last_line.as_deref(), Some("build broke"));
        assert_eq!(build.command, "sleep 2; echo build broke >&2; exit 3");

        let end = store.snapshot().sessions[0].clone();
        assert!(!end.alive);
        assert_eq!((end.shells[0].state, end.shells[0].port, end.shells[0].exit_code), (ShellState::Exited, None, None), "TaskStop");
        assert_eq!((end.shells[1].state, end.shells[1].exit_code), (ShellState::Failed, Some(3)));
        assert!(!format!("{end:?}").contains("SECRET"));
        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn replayed_task_stop_completion_and_shell_outliving_the_turn() {
        use crate::state::ShellState;
        let store = quiet_store();
        let lines: Vec<String> = fixture("shell_background_stop.jsonl").lines().map(str::to_owned).collect();
        let mut after_stop = None;
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(send_line(&store, line, 1_000 + i as u64), 204);
            if i == 14 {
                after_stop = Some(store.snapshot().sessions[0].clone());
            }
        }
        let s = after_stop.unwrap();
        assert_eq!(s.state, State::Success);
        let got: Vec<_> = s.shells.iter().map(|sh| (sh.id.as_str(), sh.state, sh.exit_code)).collect();
        assert_eq!(
            got,
            vec![
                ("bfm31czn2", ShellState::Exited, Some(0)),
                ("bv4qmnt44", ShellState::Running, None),
                ("bydeacx8e", ShellState::Exited, None),
            ]
        );
        let end = store.snapshot().sessions[0].clone();
        assert_eq!((end.shells[1].state, end.shells[1].exit_code, end.shells[1].ended_at), (ShellState::Exited, Some(0), Some(1_015)));
    }

    #[test]
    fn stop_failure_prefers_error_details() {
        let body = format!(r#"{{"session_id":"{SID}","error":"rate_limit","error_details":"429 Too Many Requests"}}"#);
        let r = parse(&Route::Hook("StopFailure".into()), body.as_bytes()).unwrap();
        assert_eq!(r.event, Some(Event::StopFailure { summary: "API error: 429 Too Many Requests".into() }));
    }

    #[test]
    fn token_survives_restart() {
        let dir = temp_dir();
        let first = spawn_on(quiet_store(), None, &dir, 0).unwrap();
        let second = spawn_on(quiet_store(), None, &dir, 0).unwrap();
        assert_eq!(first.token, second.token);
        std::fs::write(dir.join("endpoint.json"), r#"{"port":1,"token":"short"}"#).unwrap();
        let third = spawn_on(quiet_store(), None, &dir, 0).unwrap();
        assert_ne!(third.token, "short");
        assert_eq!(third.token.len(), 64);
    }

    #[test]
    fn notification_without_type_falls_back_to_message() {
        let body = format!(r#"{{"session_id":"{SID}","message":"Claude is waiting for your input"}}"#);
        let r = parse(&Route::Hook("Notification".into()), body.as_bytes()).unwrap();
        assert!(matches!(r.event, Some(Event::Notification { idle: true, .. })));
    }

    #[test]
    fn tool_targets() {
        let t = |input: Value, cwd: Option<&str>| tool_target(&input, cwd);
        assert_eq!(
            t(serde_json::json!({"pattern": "TODO", "path": "src"}), None).as_deref(),
            Some("TODO")
        );
        assert_eq!(t(serde_json::json!({"url": "https://x.dev", "prompt": "p"}), None).as_deref(), Some("https://x.dev"));
        assert_eq!(
            t(serde_json::json!({"description": "explore", "prompt": "secret"}), None).as_deref(),
            Some("explore")
        );
        assert_eq!(t(serde_json::json!({"file_path": "D:/other/a.rs"}), Some("C:/work")).as_deref(), Some("D:/other/a.rs"));
        assert_eq!(t(serde_json::json!({"file_path": "C:/work/a.rs"}), Some("C:/work/")).as_deref(), Some("a.rs"));
        assert_eq!(t(serde_json::json!({"todos": []}), None), None);
        assert_eq!(t(serde_json::json!({"command": "x".repeat(500)}), None).unwrap().chars().count(), TEXT_MAX_CHARS);
    }

    #[test]
    fn statusline_reduces_to_model_context_and_rate_limits() {
        let r = parse(&Route::Statusline, fixture("statusline.json").as_bytes()).unwrap();
        assert_eq!(r.session_id, SID);
        assert_eq!(r.event, Some(Event::Statusline { model: Some("Opus".into()), context_pct: Some(37.5) }));
        assert_eq!(r.rate_limits, Some((42.0, 18.0)));

        let bare = format!(r#"{{"session_id":"{SID}"}}"#);
        let r = parse(&Route::Statusline, bare.as_bytes()).unwrap();
        assert_eq!(r.event, Some(Event::Statusline { model: None, context_pct: None }));
        assert_eq!(r.rate_limits, None);
    }

    #[test]
    fn rejects_malformed_or_anonymous_payloads() {
        let route = Route::Hook("Stop".into());
        assert!(parse(&route, b"{nope").is_err());
        assert!(parse(&route, b"[]").is_err());
        assert!(parse(&route, br#"{"session_id":""}"#).is_err());
    }

    #[test]
    fn handle_rejects_bad_token_before_anything_else() {
        let store = quiet_store();
        let body = fixture("user_prompt_submit.json");
        assert_eq!(send(&store, &req("/hook/UserPromptSubmit", None), &body), 401);
        assert_eq!(send(&store, &req("/hook/UserPromptSubmit", Some("nope")), &body), 401);
        assert_eq!(send(&store, &req("/elsewhere", None), &body), 401);
        assert!(store.snapshot().sessions.is_empty());
    }

    #[test]
    fn handle_status_codes() {
        let store = quiet_store();
        let ok = Some(TOKEN);
        assert_eq!(send(&store, &req("/elsewhere", ok), "{}"), 404);
        let get = Request { post: false, ..req("/statusline", ok) };
        assert_eq!(send(&store, &get, "{}"), 405);
        assert_eq!(send(&store, &req("/hook/Stop", ok), "{nope"), 400);
        assert_eq!(send(&store, &req("/hook/PreCompact", ok), &fixture("stop.json")), 204);
        assert!(store.snapshot().sessions.is_empty());
    }

    #[test]
    fn handle_caps_body_at_1mb() {
        let store = quiet_store();
        let declared = Request { declared_len: Some(MAX_BODY + 1), ..req("/hook/Stop", Some(TOKEN)) };
        assert_eq!(send(&store, &declared, "{}"), 413);

        let pad = " ".repeat(MAX_BODY);
        let big = format!(r#"{{"session_id":"{SID}"}}{pad}"#);
        assert_eq!(send(&store, &req("/hook/Stop", Some(TOKEN)), &big), 413);

        let exact = format!(r#"{{"session_id":"{SID}"}}"#);
        let exact = format!("{exact}{}", " ".repeat(MAX_BODY - exact.len()));
        assert_eq!(send(&store, &req("/hook/Stop", Some(TOKEN)), &exact), 204);
    }

    #[test]
    fn handle_routes_hooks_and_statusline_into_store() {
        let store = quiet_store();
        let ok = Some(TOKEN);
        assert_eq!(send(&store, &req("/hook/PreToolUse", ok), &fixture("pre_tool_use_edit.json")), 204);
        let s = store.snapshot().sessions[0].clone();
        assert_eq!((s.id.as_str(), s.name.as_str(), s.pid), (SID, "demo", 0));
        assert_eq!(s.state, State::Working);
        assert_eq!(s.task, "Edit src\\main.ts");

        assert_eq!(send(&store, &req("/statusline", ok), &fixture("statusline.json")), 204);
        let snap = store.snapshot();
        assert_eq!(snap.sessions[0].context_pct, Some(37.5));
        assert_eq!(snap.sessions[0].model.as_deref(), Some("Opus"));
        let rl = snap.rate_limits.unwrap();
        assert_eq!((rl.five_hour_pct, rl.seven_day_pct), (42.0, 18.0));

        assert_eq!(send(&store, &req("/hook/Notification", ok), &fixture("notification_permission.json")), 204);
        assert_eq!(store.snapshot().sessions[0].state, State::Waiting);
    }

    fn post(port: u16, path: &str, token: Option<&str>, body: &str) -> u16 {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let mut head = format!(
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        if let Some(t) = token {
            head.push_str(&format!("{TOKEN_HEADER}: {t}\r\n"));
        }
        s.write_all(format!("{head}\r\n{body}").as_bytes()).unwrap();
        let mut resp = String::new();
        s.read_to_string(&mut resp).unwrap();
        resp.split_whitespace().nth(1).unwrap().parse().unwrap()
    }

    #[test]
    fn serves_over_loopback_and_writes_endpoint_file() {
        let store = quiet_store();
        let dir = temp_dir();
        let ep = spawn_on(Arc::clone(&store), None, &dir, 0).unwrap();
        let file: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("endpoint.json")).unwrap()).unwrap();
        assert_eq!(file["port"], ep.port);
        assert_eq!(file["token"], ep.token.as_str());
        assert!(!dir.join("endpoint.json.tmp").exists());

        let body = fixture("user_prompt_submit.json");
        assert_eq!(post(ep.port, "/hook/UserPromptSubmit", None, &body), 401);
        assert!(store.snapshot().sessions.is_empty());
        assert_eq!(post(ep.port, "/hook/UserPromptSubmit", Some(&ep.token), &body), 204);
        assert_eq!(store.snapshot().sessions[0].state, State::Thinking);
        let _ = std::fs::remove_dir_all(dir);
    }

    struct NoEnv;
    impl approve::Env for NoEnv {
        fn terminal_foreground(&self, _: u32, _: &str) -> bool {
            false
        }
        fn widget_hidden(&self) -> bool {
            false
        }
    }

    fn open_permission_request(port: u16, token: &str) -> TcpStream {
        let body = fixture("permission_request.json");
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let head = format!(
            "POST /hook/PermissionRequest HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\n{TOKEN_HEADER}: {token}\r\n\r\n",
            body.len()
        );
        s.write_all(format!("{head}{body}").as_bytes()).unwrap();
        s
    }

    fn wait_for(mut f: impl FnMut() -> bool) {
        for _ in 0..200 {
            if f() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("condition not reached");
    }

    #[test]
    fn permission_request_is_held_over_tcp_until_decided() {
        let store = quiet_store();
        let broker = Broker::new(Arc::clone(&store), Box::new(NoEnv), approve::HOLD_MS);
        let dir = temp_dir();
        let ep = spawn_on(Arc::clone(&store), Some(Arc::clone(&broker)), &dir, 0).unwrap();
        let mut s = open_permission_request(ep.port, &ep.token);
        wait_for(|| store.snapshot().sessions.first().is_some_and(|x| !x.pending.is_empty()));
        // Other hooks still get through while one is held.
        assert_eq!(post(ep.port, "/hook/Notification", Some(&ep.token), &fixture("notification_permission.json")), 204);
        let id = store.snapshot().sessions[0].pending[0].id.clone();
        assert_eq!(broker.decide(&id, false, Some("nope".into()), store::now_ms()), Ok(true));
        let mut resp = String::new();
        s.read_to_string(&mut resp).unwrap();
        assert!(resp.starts_with("HTTP/1.1 200"), "{resp}");
        let json: Value = serde_json::from_str(resp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(json["hookSpecificOutput"]["decision"]["message"], "nope");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn dropped_connection_is_seen_and_released() {
        let store = quiet_store();
        let broker = Broker::new(Arc::clone(&store), Box::new(NoEnv), approve::HOLD_MS);
        let dir = temp_dir();
        let ep = spawn_on(Arc::clone(&store), Some(Arc::clone(&broker)), &dir, 0).unwrap();
        let s = open_permission_request(ep.port, &ep.token);
        wait_for(|| store.snapshot().sessions.first().is_some_and(|x| !x.pending.is_empty()));
        drop(s);
        wait_for(|| {
            broker.sweep(store::now_ms());
            store.snapshot().sessions[0].pending.is_empty()
        });
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn falls_back_to_random_port_when_taken() {
        let held = TcpListener::bind("127.0.0.1:0").unwrap();
        let taken = held.local_addr().unwrap().port();
        let dir = temp_dir();
        let ep = spawn_on(quiet_store(), None, &dir, taken).unwrap();
        assert_ne!(ep.port, taken);
        assert_eq!(post(ep.port, "/hook/Stop", Some(&ep.token), &fixture("stop.json")), 204);
        let _ = std::fs::remove_dir_all(dir);
    }
}
