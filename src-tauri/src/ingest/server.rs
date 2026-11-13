//! Loopback event server for hook and statusline payloads (spec §6.5).

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use tiny_http::{Method, Response, Server};

use crate::state::Event;
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

/// Binds 127.0.0.1:47821 (random port if taken), writes `endpoint.json` into `dir` and serves on a background thread.
pub fn spawn(store: Arc<Store>, dir: &Path) -> io::Result<Endpoint> {
    spawn_on(store, dir, DEFAULT_PORT)
}

fn spawn_on(store: Arc<Store>, dir: &Path, port: u16) -> io::Result<Endpoint> {
    let server = Server::http(("127.0.0.1", port))
        .or_else(|e| {
            println!("peepthis: port {port} unavailable ({e}), using a random port");
            Server::http(("127.0.0.1", 0))
        })
        .map_err(io::Error::other)?;
    let port = server
        .server_addr()
        .to_ip()
        .map(|a| a.port())
        .ok_or_else(|| io::Error::other("event server has no IP address"))?;
    let token = new_token()?;
    write_endpoint(dir, port, &token)?;
    println!("peepthis: event server on 127.0.0.1:{port}");

    let expected = token.clone();
    std::thread::Builder::new()
        .name("event-server".into())
        .spawn(move || {
            for req in server.incoming_requests() {
                serve(&store, &expected, req);
            }
        })?;
    Ok(Endpoint { port, token })
}

fn serve(store: &Store, expected: &str, mut req: tiny_http::Request) {
    let url = req.url().to_string();
    let token = req
        .headers()
        .iter()
        .find(|h| h.field.equiv(TOKEN_HEADER))
        .map(|h| h.value.as_str().to_string());
    let meta = Request {
        post: *req.method() == Method::Post,
        url: &url,
        token: token.as_deref(),
        declared_len: req.body_length(),
    };
    let status = handle(store, expected, &meta, req.as_reader(), store::now_ms());
    let _ = req.respond(Response::empty(status));
}

/// Authorises, size-checks, parses and dispatches one request; returns the HTTP status.
pub fn handle(store: &Store, expected: &str, req: &Request, body: impl Read, now: u64) -> u16 {
    if !token_matches(expected, req.token) {
        return 401;
    }
    let Some(route) = parse_route(req.url) else { return 404 };
    if !req.post {
        return 405;
    }
    if req.declared_len.is_some_and(|n| n > MAX_BODY) {
        return 413;
    }
    let mut buf = Vec::new();
    if body.take(MAX_BODY as u64 + 1).read_to_end(&mut buf).is_err() {
        return 400;
    }
    if buf.len() > MAX_BODY {
        return 413;
    }
    match parse(&route, &buf) {
        Ok(routed) => {
            dispatch(store, routed, now);
            204
        }
        Err(e) => {
            println!("peepthis: dropped {:?} payload: {e}", route);
            400
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
            event: hook_event(name, &v, cwd.as_deref()),
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
        "PreToolUse" => Event::PreToolUse {
            tool: str_at(v, "/tool_name")?,
            target: v.get("tool_input").and_then(|i| tool_target(i, cwd)),
        },
        "PostToolUse" | "PostToolUseFailure" => Event::PostToolUse,
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
        "SubagentStop" => Event::SubagentStop,
        "SessionEnd" => Event::Ended,
        _ => return None,
    })
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

fn failure_summary(v: &Value) -> String {
    ["/error", "/error/message", "/error_message", "/message", "/reason"]
        .iter()
        .find_map(|p| str_at(v, p))
        .map(|s| clip(&s))
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
                Event::PreToolUse { tool: "Edit".into(), target: Some("src\\main.ts".into()) },
            ),
            (
                "PreToolUse",
                "pre_tool_use_bash.json",
                Event::PreToolUse { tool: "Bash".into(), target: Some("npm test".into()) },
            ),
            ("PostToolUse", "post_tool_use.json", Event::PostToolUse),
            (
                "Notification",
                "notification_permission.json",
                Event::Notification { message: "Claude needs your permission to use Bash".into(), idle: false },
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
                Event::StopFailure { summary: "API Error: 529 Overloaded".into() },
            ),
            ("SubagentStop", "subagent_stop.json", Event::SubagentStop),
            ("SessionEnd", "session_end.json", Event::Ended),
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
        let ep = spawn_on(Arc::clone(&store), &dir, 0).unwrap();
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

    #[test]
    fn falls_back_to_random_port_when_taken() {
        let held = TcpListener::bind("127.0.0.1:0").unwrap();
        let taken = held.local_addr().unwrap().port();
        let dir = temp_dir();
        let ep = spawn_on(quiet_store(), &dir, taken).unwrap();
        assert_ne!(ep.port, taken);
        assert_eq!(post(ep.port, "/hook/Stop", Some(&ep.token), &fixture("stop.json")), 204);
        let _ = std::fs::remove_dir_all(dir);
    }
}
