//! Approval broker (spec §11.1): holds `PermissionRequest` hooks, answers with the widget's decision,
//! or releases them to the terminal prompt with no decision.

use std::net::TcpStream;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};

use crate::ingest::http;
use crate::risk::{self, Tier, Verdict};
use crate::rules::Rules;
use crate::state::{ApprovalOutcome, Event, Session, State};
use crate::store::{self, Store};
use crate::trail::Outcome;

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

/// Stands in for a question's hook connection, which was answered 204 on arrival.
struct Answered;

impl Responder for Answered {
    fn reply(self: Box<Self>, _: Reply) -> bool {
        true
    }
    fn closed(&self) -> bool {
        false
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

/// One settled request, handed to the recorder (the decision trail).
pub struct Decision<'a> {
    pub session_id: &'a str,
    pub request: &'a PendingApproval,
    pub outcome: Outcome,
    pub reason: Option<&'a str>,
    pub via: Option<Release>,
    /// The rule that auto-allowed it.
    pub rule: Option<&'a str>,
    pub at: u64,
}

pub type Recorder = Box<dyn Fn(&Decision<'_>) + Send + Sync>;

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
    recorder: OnceLock<Recorder>,
    rules: OnceLock<Arc<Rules>>,
}

impl Broker {
    pub fn new(store: Arc<Store>, env: Box<dyn Env>, hold_ms: u64) -> Arc<Broker> {
        Arc::new(Broker {
            store,
            env,
            hold_ms,
            inner: Mutex::new(Inner::default()),
            wake: Condvar::new(),
            recorder: OnceLock::new(),
            rules: OnceLock::new(),
        })
    }

    pub fn set_recorder(&self, r: Recorder) {
        let _ = self.recorder.set(r);
    }

    /// Without rules nothing is ever auto-allowed.
    pub fn set_rules(&self, r: Arc<Rules>) {
        let _ = self.rules.set(r);
    }

    fn record(&self, session_id: &str, request: &PendingApproval, outcome: Outcome, reason: Option<&str>, via: Option<Release>, at: u64) {
        self.record_rule(session_id, request, outcome, reason, via, None, at);
    }

    #[allow(clippy::too_many_arguments)]
    fn record_rule(
        &self,
        session_id: &str,
        request: &PendingApproval,
        outcome: Outcome,
        reason: Option<&str>,
        via: Option<Release>,
        rule: Option<&str>,
        at: u64,
    ) {
        if let Some(r) = self.recorder.get() {
            r(&Decision { session_id, request, outcome, reason, via, rule, at });
        }
    }

    /// A confirmed rule answers at once; the request is never held or shown.
    fn auto_allow(&self, req: Request, risk: Verdict, rule: String, responder: Box<dyn Responder>, now: u64) {
        let meta = PendingApproval {
            id: format!("auto-{now}"),
            tool: req.tool,
            input: req.input,
            cwd: req.cwd,
            agent_id: req.agent_id,
            received_at: now,
            release_at: now,
            risk,
        };
        let delivered = responder.reply(Reply::Allow);
        println!("peepthis: auto-allowed by rule {rule} for session {} (delivered: {delivered})", req.session_id);
        if delivered {
            self.record_rule(&req.session_id, &meta, Outcome::AutoAllowed, None, None, Some(&rule), now);
            if let Some(r) = self.rules.get() {
                r.record_use(&rule, now);
            }
            self.store.apply(&req.session_id, Event::Approval { agent_id: meta.agent_id, outcome: ApprovalOutcome::Allowed }, now);
        } else {
            self.record(&req.session_id, &meta, Outcome::Released, None, Some(Release::Disconnected), now);
            self.abandon(req.session_id, meta.agent_id, now);
        }
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
        let risk = risk::classify(&req.tool, &req.input, &req.cwd);
        // AV-50: the terminal shows the question at once; the card stays until the session leaves `waiting`.
        let responder: Box<dyn Responder> = if risk.tier == Tier::Question {
            responder.reply(Reply::Release);
            if self.env.widget_hidden() {
                return None;
            }
            Box::new(Answered)
        } else {
            responder
        };
        let rule = (!matches!(risk.tier, Tier::Destructive | Tier::Question))
            .then(|| self.rules.get().and_then(|r| r.auto_rule(&req.tool, &req.input, &req.cwd)))
            .flatten();
        if let Some(rule) = rule {
            self.auto_allow(req, risk, rule, responder, now);
            return None;
        }
        if self.env.widget_hidden() {
            responder.reply(Reply::Release);
            return None;
        }
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
        let Some(h) = self.take(|h| h.meta.id == id && h.meta.risk.tier != Tier::Question).pop() else { return Ok(false) };
        let reply = if allow {
            Reply::Allow
        } else {
            Reply::Deny(reason.map(|r| r.trim().to_string()).filter(|r| !r.is_empty()).unwrap_or_else(|| DEFAULT_DENY.into()))
        };
        let (session_id, agent_id, meta) = (h.session_id.clone(), h.meta.agent_id.clone(), h.meta.clone());
        let reason = match &reply {
            Reply::Deny(r) => Some(r.clone()),
            _ => None,
        };
        let delivered = h.responder.reply(reply);
        self.sync(&session_id);
        println!("peepthis: approval {id} {} (delivered: {delivered})", if allow { "allowed" } else { "denied" });
        if delivered {
            let outcome = if allow { Outcome::Allowed } else { Outcome::Denied };
            self.record(&session_id, &meta, outcome, reason.as_deref(), None, now);
            if let Some(r) = self.rules.get() {
                if allow {
                    r.note_allow(&session_id, &meta.tool, &meta.input, &meta.cwd, now);
                } else {
                    r.note_deny(&meta.tool, &meta.input, &meta.cwd);
                }
            }
        } else {
            self.record(&session_id, &meta, Outcome::Released, None, Some(Release::Disconnected), now);
        }
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
        let candidates: Vec<(String, String, Option<String>, u64, bool, bool)> = {
            let inner = self.inner.lock().unwrap();
            inner
                .held
                .iter()
                .map(|h| {
                    let question = h.meta.risk.tier == Tier::Question;
                    (h.meta.id.clone(), h.session_id.clone(), h.meta.agent_id.clone(), h.meta.release_at, h.responder.closed(), question)
                })
                .collect()
        };
        let mut foreground: Vec<(String, bool)> = Vec::new();
        let mut verdicts: Vec<(String, Release)> = Vec::new();
        for (id, sid, agent, release_at, closed, question) in candidates {
            let session = find(&sid);
            let why = if closed {
                Some(Release::Disconnected)
            } else if hidden {
                Some(Release::Hidden)
            } else if session.is_none_or(|s| !s.alive) {
                Some(Release::SessionGone)
            } else if !waiting(session.unwrap(), agent.as_deref()) {
                Some(Release::LeftWaiting)
            } else if question {
                None
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
            if h.meta.risk.tier != Tier::Question {
                self.record(&sid, &h.meta, Outcome::Released, None, Some(why), now);
            }
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
                    title: None,
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
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy/hooks").join(file);
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
    fn every_settled_request_reaches_the_recorder() {
        let r = rig();
        let log: Arc<Mutex<Vec<(String, Outcome, Option<String>, Option<Release>)>>> = Arc::default();
        let sink = Arc::clone(&log);
        r.broker.set_recorder(Box::new(move |d: &Decision<'_>| {
            sink.lock().unwrap().push((d.request.tool.clone(), d.outcome, d.reason.map(str::to_owned), d.via));
        }));
        let (a, _) = r.request("permission_request.json", T0);
        r.broker.decide(&a, true, None, T0 + 1).unwrap();
        let (b, _) = r.request("permission_request.json", T0 + 2);
        r.broker.decide(&b, false, Some("nope".into()), T0 + 3).unwrap();
        r.request("permission_request.json", T0 + 4);
        r.broker.sweep(T0 + 4 + HOLD_MS);
        let (_, gone) = r.request("permission_request.json", T0 + HOLD_MS + 10);
        gone.closed.store(true, Ordering::SeqCst);
        r.broker.sweep(T0 + HOLD_MS + 20);
        let log = log.lock().unwrap();
        let outcomes: Vec<_> = log.iter().map(|(_, o, reason, via)| (*o, reason.clone(), *via)).collect();
        assert_eq!(
            outcomes,
            vec![
                (Outcome::Allowed, None, None),
                (Outcome::Denied, Some("nope".into()), None),
                (Outcome::Released, None, Some(Release::Window)),
                (Outcome::Released, None, Some(Release::Disconnected)),
            ]
        );
    }

    type Log = Arc<Mutex<Vec<(Outcome, Option<String>)>>>;

    fn with_rules(r: &Rig) -> (crate::rules::tests::TempDir, Arc<Rules>, Log) {
        let d = crate::rules::tests::temp();
        let rules = Rules::open(&d.0);
        r.broker.set_rules(Arc::clone(&rules));
        let log: Log = Arc::default();
        let sink = Arc::clone(&log);
        r.broker.set_recorder(Box::new(move |d: &Decision<'_>| {
            sink.lock().unwrap().push((d.outcome, d.rule.map(str::to_owned)));
        }));
        (d, rules, log)
    }

    #[test]
    fn three_allows_offer_a_rule_and_an_accepted_rule_answers_without_holding() {
        let r = rig();
        let (_d, rules, log) = with_rules(&r);
        for i in 0..3 {
            assert!(rules.view().offers.is_empty(), "no offer before allow {}", i + 1);
            let (id, _) = r.request("permission_request_bash.json", T0 + i * 10);
            r.broker.decide(&id, true, None, T0 + i * 10 + 1).unwrap();
        }
        let offer = rules.view().offers[0].clone();
        assert_eq!((offer.session_id.as_str(), offer.shape.pattern.as_str()), (SID, "mkdir zz-test"));
        let rule = rules.answer(&offer.id, crate::rules::Answer::Yes, T0 + 50).unwrap().rules[0].id.clone();

        let v = payload("permission_request_bash.json");
        r.store.apply(SID, Event::UserPromptSubmit, T0 + 60);
        let probe = Arc::new(Probe::default());
        assert_eq!(r.broker.hold(Request::from_payload(&v).unwrap(), Box::new(Fake(Arc::clone(&probe))), T0 + 60), None);
        assert_eq!(replies(&probe), vec![Reply::Allow]);
        assert!(r.session().pending.is_empty());
        assert_eq!(log.lock().unwrap().last().cloned(), Some((Outcome::AutoAllowed, Some(rule.clone()))));
        assert_eq!(rules.view().rules[0].uses, 1);

        rules.revoke(&rule).unwrap();
        let (_, after) = r.request("permission_request_bash.json", T0 + 70);
        assert!(replies(&after).is_empty(), "held again once revoked");
        assert_eq!(r.session().pending.len(), 1);
    }

    #[test]
    fn a_rule_never_lets_a_destructive_request_through() {
        let r = rig();
        let (d, _rules, log) = with_rules(&r);
        let mut v = payload("permission_request_bash.json");
        v["tool_input"]["command"] = "git push --force".into();
        let shape = crate::rules::shape_of("Bash", &v["tool_input"], "C:\\work\\demo").unwrap();
        let book = json!({ "version": 1, "rules": [{ "id": "r1", "cwd": shape.cwd, "tool": "Bash", "pattern": shape.pattern, "created": T0 }] });
        std::fs::write(d.0.join("rules.json"), book.to_string()).unwrap();
        let probe = Arc::new(Probe::default());
        let held = r.broker.hold(Request::from_payload(&v).unwrap(), Box::new(Fake(Arc::clone(&probe))), T0);
        assert!(held.is_some());
        assert!(replies(&probe).is_empty());
        assert!(log.lock().unwrap().is_empty());
    }

    #[test]
    fn question_is_released_at_once_and_its_card_stays_until_answered() {
        let r = rig();
        let (_d, rules, log) = with_rules(&r);
        let (id, probe) = r.request("permission_request_question.json", T0);
        assert_eq!(replies(&probe), vec![Reply::Release], "the terminal shows the question with no delay");
        let s = r.session();
        assert_eq!((s.state, s.task.as_str()), (State::Waiting, "Which colour?"));
        assert_eq!(s.pending[0].risk.tier, Tier::Question);

        r.foreground.store(true, Ordering::SeqCst);
        r.broker.sweep(T0 + HOLD_MS * 10);
        assert_eq!(r.session().pending.len(), 1, "no window and no foreground release");
        assert_eq!(r.broker.decide(&id, true, None, T0 + 1), Ok(false), "the widget never answers a question");
        assert_eq!(r.session().pending.len(), 1);

        r.store.apply(SID, Event::PostToolUse, T0 + 5);
        r.broker.sweep(T0 + 10);
        assert!(r.session().pending.is_empty());
        assert_eq!(replies(&probe), vec![Reply::Release]);
        assert!(log.lock().unwrap().is_empty(), "no trail entry");
        assert!(rules.view().offers.is_empty());
    }

    #[test]
    fn question_shows_no_card_while_the_widget_is_hidden() {
        let r = rig();
        r.hidden.store(true, Ordering::SeqCst);
        let probe = Arc::new(Probe::default());
        let v = payload("permission_request_question.json");
        assert_eq!(r.broker.hold(Request::from_payload(&v).unwrap(), Box::new(Fake(Arc::clone(&probe))), T0), None);
        assert_eq!(replies(&probe), vec![Reply::Release]);
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
