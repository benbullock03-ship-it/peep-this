//! Approval broker (spec §11.1): holds `PermissionRequest` hooks, answers with the widget's decision,
//! or releases them to the terminal prompt with no decision.

use std::net::TcpStream;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};

use crate::ingest::http;
use crate::risk::{self, Verdict};
use crate::state::{ApprovalOutcome, Event, Session, State};
use crate::store::{self, Store};

pub const HOLD_MS: u64 = 60_000;
/// After Claude Code drops a held request (the terminal answered), a `PostToolUse` within this window means "yes".
pub const ABANDON_GRACE_MS: u64 = 3_000;
const POLL: Duration = Duration::from_millis(250);
const DEFAULT_DENY: &str = "Denied from Peep This.";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingApproval {
    pub id: String,
    pub tool: String,
    /// The tool input as sent; the card needs the command, path and old/new strings.
    pub input: Value,
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub received_at: u64,
    pub release_at: u64,
    pub risk: Verdict,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    Allow,
    Deny(String),
    /// 204: no decision, the terminal prompt carries on.
    Release,
}

impl Reply {
    pub fn body(&self) -> Option<String> {
        let decision = match self {
            Reply::Allow => json!({ "behavior": "allow" }),
            Reply::Deny(message) => json!({ "behavior": "deny", "message": message }),
            Reply::Release => return None,
        };
        Some(json!({ "hookSpecificOutput": { "hookEventName": "PermissionRequest", "decision": decision } }).to_string())
    }
}

pub trait Responder: Send {
    /// False when the connection was already gone.
    fn reply(self: Box<Self>, reply: Reply) -> bool;
    fn closed(&self) -> bool;
}

pub struct TcpResponder(pub TcpStream);

impl Responder for TcpResponder {
    fn reply(self: Box<Self>, reply: Reply) -> bool {
        if http::peer_closed(&self.0) {
            return false;
        }
        let status = if reply == Reply::Release { 204 } else { 200 };
        http::respond(&self.0, status, reply.body().as_deref()).is_ok()
    }
    fn closed(&self) -> bool {
        http::peer_closed(&self.0)
    }
}

/// OS lookups the sweep needs; faked in tests.
pub trait Env: Send + Sync {
    fn terminal_foreground(&self, pid: u32, cwd: &str) -> bool;
    fn widget_hidden(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    Window,
    Foreground,
    LeftWaiting,
    SessionGone,
    Disconnected,
    Hidden,
    Quit,
}

/// What the event server hands over for one `PermissionRequest`.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub session_id: String,
    pub cwd: String,
    pub tool: String,
    pub input: Value,
    pub agent_id: Option<String>,
}

impl Request {
    pub fn from_payload(v: &Value) -> Option<Request> {
        let s = |p: &str| v.pointer(p).and_then(Value::as_str).map(str::to_owned);
        Some(Request {
            session_id: s("/session_id").filter(|x| !x.trim().is_empty())?,
            cwd: s("/cwd").unwrap_or_default(),
            tool: s("/tool_name").unwrap_or_default(),
            input: v.get("tool_input").cloned().unwrap_or(Value::Null),
            agent_id: s("/agent_id").filter(|x| !x.trim().is_empty()),
        })
    }
}

struct Held {
    session_id: String,
    meta: PendingApproval,
    responder: Box<dyn Responder>,
}

struct Abandoned {
    session_id: String,
    agent_id: Option<String>,
    closed_at: u64,
}

#[derive(Default)]
struct Inner {
    held: Vec<Held>,
    abandoned: Vec<Abandoned>,
    seq: u64,
}

pub struct Broker {
    store: Arc<Store>,
    env: Box<dyn Env>,
    hold_ms: u64,
    inner: Mutex<Inner>,
    wake: Condvar,
}

impl Broker {
    pub fn new(store: Arc<Store>, env: Box<dyn Env>, hold_ms: u64) -> Arc<Broker> {
        Arc::new(Broker { store, env, hold_ms, inner: Mutex::new(Inner::default()), wake: Condvar::new() })
    }

    pub fn spawn(self: &Arc<Self>) {
        let b = Arc::clone(self);
        std::thread::Builder::new()
            .name("approvals".into())
            .spawn(move || loop {
                {
                    let mut inner = b.inner.lock().unwrap();
                    while inner.held.is_empty() && inner.abandoned.is_empty() {
                        inner = b.wake.wait(inner).unwrap();
                    }
                }
                std::thread::sleep(POLL);
                b.sweep(store::now_ms());
            })
            .expect("spawn approval broker");
    }

    /// Takes over the hook's connection. Released at once when the widget is hidden.
    pub fn hold(&self, req: Request, responder: Box<dyn Responder>, now: u64) -> Option<String> {
        if self.env.widget_hidden() {
            responder.reply(Reply::Release);
            return None;
        }
        let risk = risk::classify(&req.tool, &req.input, &req.cwd);
        let session_id = req.session_id.clone();
        let id = {
            let mut inner = self.inner.lock().unwrap();
            inner.seq += 1;
            let id = format!("ap-{now}-{}", inner.seq);
            let meta = PendingApproval {
                id: id.clone(),
                tool: req.tool,
                input: req.input,
                cwd: req.cwd,
                agent_id: req.agent_id,
                received_at: now,
                release_at: now + self.hold_ms,
                risk,
            };
            inner.held.push(Held { session_id: session_id.clone(), meta, responder });
            self.wake.notify_one();
            id
        };
        self.sync(&session_id);
        println!("peepthis: holding approval {id} for session {session_id}");
        Some(id)
    }

    /// `Ok(false)`: unknown id (already decided or released), or Claude Code had already dropped it.
    pub fn decide(&self, id: &str, allow: bool, reason: Option<String>, now: u64) -> Result<bool, String> {
        let Some(h) = self.take(|h| h.meta.id == id).pop() else { return Ok(false) };
        let reply = if allow {
            Reply::Allow
        } else {
            Reply::Deny(reason.map(|r| r.trim().to_string()).filter(|r| !r.is_empty()).unwrap_or_else(|| DEFAULT_DENY.into()))
        };
        let (session_id, agent_id) = (h.session_id.clone(), h.meta.agent_id.clone());
        let delivered = h.responder.reply(reply);
        self.sync(&session_id);
        println!("peepthis: approval {id} {} (delivered: {delivered})", if allow { "allowed" } else { "denied" });
        if delivered {
            let outcome = if allow { ApprovalOutcome::Allowed } else { ApprovalOutcome::Denied };
            self.store.apply(&session_id, Event::Approval { agent_id, outcome }, now);
        } else {
            self.abandon(session_id, agent_id, now);
        }
        Ok(delivered)
    }

    pub fn release_all(&self, why: Release) {
        self.release(|_| true, why);
    }

    /// One pass over everything held; the loop runs it every `POLL` while anything is pending.
    pub fn sweep(&self, now: u64) {
        let snap = self.store.snapshot();
        let find = |id: &str| snap.sessions.iter().find(|s| s.id == id);
        let hidden = self.env.widget_hidden();
        let candidates: Vec<(String, String, Option<String>, u64, bool)> = {
            let inner = self.inner.lock().unwrap();
            inner
                .held
                .iter()
                .map(|h| {
                    (h.meta.id.clone(), h.session_id.clone(), h.meta.agent_id.clone(), h.meta.release_at, h.responder.closed())
                })
                .collect()
        };
        let mut foreground: Vec<(String, bool)> = Vec::new();
        let mut verdicts: Vec<(String, Release)> = Vec::new();
        for (id, sid, agent, release_at, closed) in candidates {
            let session = find(&sid);
            let why = if closed {
                Some(Release::Disconnected)
            } else if hidden {
                Some(Release::Hidden)
            } else if session.is_none_or(|s| !s.alive) {
                Some(Release::SessionGone)
            } else if !waiting(session.unwrap(), agent.as_deref()) {
                Some(Release::LeftWaiting)
            } else if now >= release_at {
                Some(Release::Window)
            } else {
                let s = session.unwrap();
                let fg = match foreground.iter().find(|(k, _)| *k == sid) {
                    Some((_, v)) => *v,
                    None => {
                        let v = s.pid != 0 && self.env.terminal_foreground(s.pid, &s.cwd);
                        foreground.push((sid.clone(), v));
                        v
                    }
                };
                fg.then_some(Release::Foreground)
            };
            if let Some(w) = why {
                verdicts.push((id, w));
            }
        }
        for (id, why) in verdicts {
            self.release(|h| h.meta.id == id, why);
        }
        self.settle_abandoned(now);
    }

    fn release(&self, pick: impl Fn(&Held) -> bool, why: Release) {
        let now = store::now_ms();
        for h in self.take(pick) {
            let (sid, agent) = (h.session_id.clone(), h.meta.agent_id.clone());
            println!("peepthis: released approval {} ({why:?})", h.meta.id);
            if why == Release::Disconnected {
                self.abandon(sid.clone(), agent, now);
            } else {
                h.responder.reply(Reply::Release);
            }
            self.sync(&sid);
        }
    }

    fn take(&self, pick: impl Fn(&Held) -> bool) -> Vec<Held> {
        let mut inner = self.inner.lock().unwrap();
        let (out, keep) = std::mem::take(&mut inner.held).into_iter().partition(|h| pick(h));
        inner.held = keep;
        out
    }

    fn abandon(&self, session_id: String, agent_id: Option<String>, closed_at: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.abandoned.push(Abandoned { session_id, agent_id, closed_at });
        self.wake.notify_one();
    }

    /// The terminal took a dropped request. No `PostToolUse` by now means it said no, and no `Stop` will come.
    fn settle_abandoned(&self, now: u64) {
        let due: Vec<Abandoned> = {
            let mut inner = self.inner.lock().unwrap();
            let (due, keep) = std::mem::take(&mut inner.abandoned)
                .into_iter()
                .partition(|a| now >= a.closed_at + ABANDON_GRACE_MS);
            inner.abandoned = keep;
            due
        };
        if due.is_empty() {
            return;
        }
        let snap = self.store.snapshot();
        for a in due {
            let Some(s) = snap.sessions.iter().find(|s| s.id == a.session_id) else { continue };
            let since = match &a.agent_id {
                Some(id) => s.agents.iter().find(|x| x.id == *id).filter(|x| x.state == State::Waiting).map(|x| x.state_since),
                None => (s.state == State::Waiting).then_some(s.state_since),
            };
            if since.is_some_and(|t| t <= a.closed_at) {
                self.store.apply(&a.session_id, Event::Approval { agent_id: a.agent_id, outcome: ApprovalOutcome::Abandoned }, now);
            }
        }
    }

    fn sync(&self, session_id: &str) {
        let list: Vec<PendingApproval> = {
            let inner = self.inner.lock().unwrap();
            inner.held.iter().filter(|h| h.session_id == session_id).map(|h| h.meta.clone()).collect()
        };
        self.store.set_pending(session_id, list);
    }
}

/// A subagent's request follows that agent's state; the session's own request follows the session.
fn waiting(s: &Session, agent_id: Option<&str>) -> bool {
    match agent_id {
        Some(id) => s.agents.iter().any(|a| a.id == id && a.ended_at.is_none() && a.state == State::Waiting),
        None => s.state == State::Waiting,
    }
}

#[tauri::command]
pub fn decide_approval(
    request_id: String,
    decision: String,
    reason: Option<String>,
    broker: tauri::State<'_, Arc<Broker>>,
) -> Result<bool, String> {
    let allow = match decision.as_str() {
        "allow" => true,
        "deny" => false,
        other => return Err(format!("unknown decision {other:?}")),
    };
    broker.decide(&request_id, allow, reason, store::now_ms())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{SessionInfo, SourceEvent};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};

    const SID: &str = "aaaaaaaa-1111-4222-8333-444444444444";
    const T0: u64 = 1_000_000;

    #[derive(Default)]
    struct Probe {
        replies: Mutex<Vec<Reply>>,
        closed: AtomicBool,
    }

    struct Fake(Arc<Probe>);

    impl Responder for Fake {
        fn reply(self: Box<Self>, reply: Reply) -> bool {
            if self.0.closed.load(Ordering::SeqCst) {
                return false;
            }
            self.0.replies.lock().unwrap().push(reply);
            true
        }
        fn closed(&self) -> bool {
            self.0.closed.load(Ordering::SeqCst)
        }
    }

    #[derive(Default)]
    struct FakeEnv {
        foreground: Arc<AtomicBool>,
        hidden: Arc<AtomicBool>,
    }

    impl Env for FakeEnv {
        fn terminal_foreground(&self, _pid: u32, _cwd: &str) -> bool {
            self.foreground.load(Ordering::SeqCst)
        }
        fn widget_hidden(&self) -> bool {
            self.hidden.load(Ordering::SeqCst)
        }
    }

    struct Rig {
        store: Arc<Store>,
        broker: Arc<Broker>,
        foreground: Arc<AtomicBool>,
        hidden: Arc<AtomicBool>,
    }

    fn rig() -> Rig {
        let store = Store::new(Box::new(|_| {}));
        store.ingest(
            SourceEvent::Upsert {
                info: SessionInfo {
                    id: SID.into(),
                    pid: 100,
                    name: "demo".into(),
                    cwd: "C:\\work\\demo".into(),
                    kind: "interactive".into(),
                    entrypoint: "cli".into(),
                },
                busy: None,
            },
            T0,
        );
        let env = FakeEnv::default();
        let (foreground, hidden) = (Arc::clone(&env.foreground), Arc::clone(&env.hidden));
        let broker = Broker::new(Arc::clone(&store), Box::new(env), HOLD_MS);
        Rig { store, broker, foreground, hidden }
    }

    fn payload(file: &str) -> Value {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hooks").join(file);
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    impl Rig {
        fn session(&self) -> Session {
            self.store.snapshot().sessions.into_iter().find(|s| s.id == SID).unwrap()
        }

        /// What the event server does: apply the hook's event, then hand the connection over.
        fn request(&self, file: &str, t: u64) -> (String, Arc<Probe>) {
            let v = payload(file);
            let routed = crate::ingest::server::parse(
                &crate::ingest::server::Route::Hook("PermissionRequest".into()),
                v.to_string().as_bytes(),
            )
            .unwrap();
            self.store.apply_or_placeholder(SID, routed.cwd.as_deref(), routed.event.unwrap(), t);
            let probe = Arc::new(Probe::default());
            let id = self.broker.hold(Request::from_payload(&v).unwrap(), Box::new(Fake(Arc::clone(&probe))), t).unwrap();
            (id, probe)
        }
    }

    fn replies(p: &Probe) -> Vec<Reply> {
        p.replies.lock().unwrap().clone()
    }

    #[test]
    fn pending_shows_in_snapshot_with_input_and_risk() {
        let r = rig();
        let (id, _) = r.request("permission_request_bash.json", T0);
        let s = r.session();
        assert_eq!(s.state, State::Waiting);
        assert_eq!(s.pending.len(), 1);
        let p = &s.pending[0];
        assert_eq!((p.id.as_str(), p.tool.as_str(), p.release_at), (id.as_str(), "Bash", T0 + HOLD_MS));
        assert!(p.input.get("command").is_some());
        assert_eq!(p.agent_id, None);
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["pending"][0]["risk"]["tier"], serde_json::to_value(p.risk.tier).unwrap());
        assert!(v["pending"][0]["releaseAt"].is_u64());
    }

    #[test]
    fn allow_answers_the_hook_and_moves_on() {
        let r = rig();
        let (id, probe) = r.request("permission_request.json", T0);
        assert_eq!(r.broker.decide(&id, true, None, T0 + 1_000), Ok(true));
        assert_eq!(replies(&probe), vec![Reply::Allow]);
        let s = r.session();
        assert!(s.pending.is_empty());
        assert_eq!(s.state, State::Working);
    }

    #[test]
    fn deny_carries_the_reason_and_leaves_waiting() {
        let r = rig();
        let (id, probe) = r.request("permission_request.json", T0);
        assert_eq!(r.broker.decide(&id, false, Some(" not that file ".into()), T0 + 1_000), Ok(true));
        assert_eq!(replies(&probe), vec![Reply::Deny("not that file".into())]);
        assert_eq!(r.session().state, State::Thinking);
    }

    #[test]
    fn deny_without_reason_uses_a_default_message() {
        let r = rig();
        let (id, probe) = r.request("permission_request.json", T0);
        r.broker.decide(&id, false, Some("  ".into()), T0).unwrap();
        assert_eq!(replies(&probe), vec![Reply::Deny(DEFAULT_DENY.into())]);
    }

    #[test]
    fn reply_bodies_match_the_verified_contract() {
        let fixture = |f: &str| -> Value { payload(f) };
        let allow: Value = serde_json::from_str(&Reply::Allow.body().unwrap()).unwrap();
        assert_eq!(allow, fixture("permission_response_allow.json"));
        let deny: Value = serde_json::from_str(&Reply::Deny("x".into()).body().unwrap()).unwrap();
        let mut want = fixture("permission_response_deny.json");
        want["hookSpecificOutput"]["decision"]["message"] = "x".into();
        assert_eq!(deny, want);
        assert_eq!(Reply::Release.body(), None);
    }

    #[test]
    fn window_end_releases_to_the_terminal() {
        let r = rig();
        let (_, probe) = r.request("permission_request.json", T0);
        r.broker.sweep(T0 + HOLD_MS - 1);
        assert!(replies(&probe).is_empty());
        r.broker.sweep(T0 + HOLD_MS);
        assert_eq!(replies(&probe), vec![Reply::Release]);
        let s = r.session();
        assert!(s.pending.is_empty());
        assert_eq!(s.state, State::Waiting, "the terminal prompt still needs an answer");
    }

    #[test]
    fn focused_terminal_releases_at_once() {
        let r = rig();
        let (_, probe) = r.request("permission_request.json", T0);
        r.foreground.store(true, Ordering::SeqCst);
        r.broker.sweep(T0 + 10);
        assert_eq!(replies(&probe), vec![Reply::Release]);
    }

    #[test]
    fn hidden_widget_releases_and_holds_nothing_new() {
        let r = rig();
        let (_, probe) = r.request("permission_request.json", T0);
        r.hidden.store(true, Ordering::SeqCst);
        r.broker.sweep(T0 + 10);
        assert_eq!(replies(&probe), vec![Reply::Release]);
        let p2 = Arc::new(Probe::default());
        let held = r.broker.hold(Request::from_payload(&payload("permission_request.json")).unwrap(), Box::new(Fake(Arc::clone(&p2))), T0);
        assert_eq!((held, replies(&p2)), (None, vec![Reply::Release]));
    }

    #[test]
    fn session_death_releases() {
        let r = rig();
        let (_, probe) = r.request("permission_request.json", T0);
        r.store.ingest(SourceEvent::Ended { id: SID.into() }, T0 + 5);
        r.broker.sweep(T0 + 10);
        assert_eq!(replies(&probe), vec![Reply::Release]);
        assert!(r.session().pending.is_empty());
    }

    #[test]
    fn leaving_waiting_by_another_event_releases() {
        let r = rig();
        let (_, probe) = r.request("permission_request.json", T0);
        r.store.apply(SID, Event::PostToolUse, T0 + 5);
        r.broker.sweep(T0 + 10);
        assert_eq!(replies(&probe), vec![Reply::Release]);
    }

    #[test]
    fn quit_releases_everything() {
        let r = rig();
        let (_, a) = r.request("permission_request.json", T0);
        let (_, b) = r.request("permission_request_subagent.json", T0);
        r.broker.release_all(Release::Quit);
        assert_eq!((replies(&a), replies(&b)), (vec![Reply::Release], vec![Reply::Release]));
        assert!(r.session().pending.is_empty());
    }

    #[test]
    fn double_decide_and_decide_after_release_are_refused() {
        let r = rig();
        let (id, probe) = r.request("permission_request.json", T0);
        assert_eq!(r.broker.decide(&id, true, None, T0), Ok(true));
        assert_eq!(r.broker.decide(&id, false, None, T0), Ok(false));
        assert_eq!(replies(&probe), vec![Reply::Allow]);

        let (id2, probe2) = r.request("permission_request.json", T0 + 100);
        r.broker.sweep(T0 + 100 + HOLD_MS);
        assert_eq!(r.broker.decide(&id2, true, None, T0 + 100 + HOLD_MS), Ok(false));
        assert_eq!(replies(&probe2), vec![Reply::Release]);
        assert_eq!(r.broker.decide("ap-nope", true, None, T0), Ok(false));
    }

    #[test]
    fn terminal_answer_drops_the_card_then_no_posttooluse_means_no() {
        let r = rig();
        let (id, probe) = r.request("permission_request.json", T0);
        probe.closed.store(true, Ordering::SeqCst);
        r.broker.sweep(T0 + 1_000);
        assert!(replies(&probe).is_empty(), "no reply to a dropped connection");
        assert!(r.session().pending.is_empty());
        assert_eq!(r.broker.decide(&id, true, None, T0 + 1_100), Ok(false));
        r.broker.sweep(T0 + 1_000 + ABANDON_GRACE_MS - 1);
        assert_eq!(r.session().state, State::Waiting);
        r.broker.sweep(store::now_ms() + ABANDON_GRACE_MS);
        assert_eq!(r.session().state, State::Idle);
    }

    #[test]
    fn terminal_yes_is_left_to_posttooluse() {
        let r = rig();
        let (_, probe) = r.request("permission_request.json", T0);
        probe.closed.store(true, Ordering::SeqCst);
        r.broker.sweep(T0 + 1_000);
        r.store.apply(SID, Event::PostToolUse, store::now_ms() + 1);
        r.broker.sweep(store::now_ms() + ABANDON_GRACE_MS + 10);
        assert_eq!(r.session().state, State::Thinking);
    }

    #[test]
    fn subagent_request_follows_its_agent_not_the_parent() {
        let r = rig();
        r.store.apply(SID, Event::SubagentStart { agent_id: "a0123456789abcdef".into(), agent_type: "general-purpose".into() }, T0);
        let (_, sub) = r.request("permission_request_subagent.json", T0);
        let (main, _) = r.request("permission_request.json", T0 + 1);
        let s = r.session();
        assert_eq!(s.pending.len(), 2);
        assert_eq!(s.pending[0].agent_id.as_deref(), Some("a0123456789abcdef"));
        r.broker.decide(&main, false, None, T0 + 2).unwrap();
        r.broker.sweep(T0 + 3);
        assert!(replies(&sub).is_empty(), "the subagent's request stays held");
        assert_eq!(r.session().pending.len(), 1);
    }

    #[test]
    fn subagent_allow_resumes_the_agent() {
        let r = rig();
        r.store.apply(SID, Event::UserPromptSubmit, T0);
        r.store.apply(SID, Event::SubagentStart { agent_id: "a0123456789abcdef".into(), agent_type: "general-purpose".into() }, T0);
        let (id, probe) = r.request("permission_request_subagent.json", T0 + 1);
        assert_eq!(r.session().state, State::Waiting);
        r.broker.decide(&id, true, None, T0 + 2).unwrap();
        assert_eq!(replies(&probe), vec![Reply::Allow]);
        let s = r.session();
        assert_eq!(s.state, State::Thinking, "parent returns to what it was doing");
        assert_eq!(s.agents[0].state, State::Working);
    }
}
