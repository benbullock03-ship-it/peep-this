//! AV-14: toast on entering `waiting`, with de-duplication (spec §3.6).

use std::collections::HashMap;
use std::sync::Mutex;

use crate::state::{Session, State};

pub const DEDUP_MS: u64 = 30_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WidgetView {
    pub focused: bool,
    pub hidden: bool,
    /// Content taller than the capped window, so some rows may be scrolled out of view.
    pub overflowing: bool,
}

impl WidgetView {
    /// Without overflow every row fits in the window, so a focused, shown widget shows every row.
    fn row_visible(&self) -> bool {
        self.focused && !self.hidden && !self.overflowing
    }
}

pub fn entered_waiting(before: Option<&Session>, after: &Session) -> bool {
    after.alive
        && after.state == State::Waiting
        && before.is_none_or(|b| !b.alive || b.state != State::Waiting)
}

pub fn should_toast(last_toast: Option<u64>, now: u64, view: WidgetView) -> bool {
    if view.row_visible() {
        return false;
    }
    last_toast.is_none_or(|t| now.saturating_sub(t) >= DEDUP_MS)
}

/// The waiting alert fires on the hook's state event, just before the broker records the held request.
pub const SHOW_DELAY_MS: u64 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastContent {
    pub title: String,
    pub body: String,
    pub hint: Option<String>,
    /// (label, activation argument)
    pub buttons: Vec<(&'static str, String)>,
    /// The held request this toast is about, if any.
    pub request: Option<String>,
}

/// AV-25: buttons only for a `safe` request; other tiers point at the inbox.
pub fn toast_content(s: &Session) -> ToastContent {
    let req = s.pending.iter().max_by(|a, b| (a.received_at, &a.id).cmp(&(b.received_at, &b.id)));
    let mut c = ToastContent { title: s.name.clone(), body: s.task.clone(), hint: None, buttons: Vec::new(), request: None };
    if let Some(p) = req {
        c.request = Some(p.id.clone());
        if p.risk.tier == crate::risk::Tier::Safe {
            c.buttons = vec![("Allow", format!("allow:{}", p.id)), ("Deny", format!("deny:{}", p.id))];
        } else {
            c.hint = Some(match crate::hotkey::registered() {
                Some(key) => format!("Review it in Peep This ({key})"),
                None => "Review it in Peep This".into(),
            });
        }
    }
    c
}

#[derive(Debug, PartialEq, Eq)]
pub enum Click {
    Focus,
    Decide { id: String, allow: bool },
    Ignore,
}

/// `action` is the button argument (None for a body click). A toast whose request has resolved does nothing.
pub fn click_action(action: Option<&str>, request: Option<&str>, pending: impl Fn(&str) -> bool) -> Click {
    match action.map(|a| a.split_once(':')) {
        Some(Some((verb @ ("allow" | "deny"), id))) if pending(id) => {
            Click::Decide { id: id.to_string(), allow: verb == "allow" }
        }
        Some(_) => Click::Ignore,
        None if request.is_some_and(|id| !pending(id)) => Click::Ignore,
        None => Click::Focus,
    }
}

pub type Show = Box<dyn Fn(&Session) + Send + Sync>;
pub type View = Box<dyn Fn() -> WidgetView + Send + Sync>;

pub struct Notifier {
    last: Mutex<HashMap<String, u64>>,
    view: View,
    show: Show,
}

impl Notifier {
    pub fn new(view: View, show: Show) -> Notifier {
        Notifier { last: Mutex::new(HashMap::new()), view, show }
    }

    /// Returns whether a toast was shown.
    pub fn alert(&self, s: &Session, now: u64) -> bool {
        {
            let mut last = self.last.lock().unwrap();
            last.retain(|_, t| now.saturating_sub(*t) < DEDUP_MS);
            if !should_toast(last.get(&s.id).copied(), now, (self.view)()) {
                return false;
            }
            last.insert(s.id.clone(), now);
        }
        (self.show)(s);
        true
    }
}

/// Shows the toast on the calling thread; `on_click` gets the button argument (None for the body) while the widget runs.
#[cfg(windows)]
pub fn toast(bundle_id: &str, c: &ToastContent, on_click: impl Fn(Option<String>) + Send + 'static) {
    use tauri_winrt_notification::{Duration, Sound, Toast};
    // Dev builds have no installed Start-menu shortcut carrying the bundle AUMID, so Windows would drop their toasts.
    let app_id = if cfg!(debug_assertions) { Toast::POWERSHELL_APP_ID.to_string() } else { bundle_id.to_string() };
    let mut t = Toast::new(&app_id).title(&c.title).text1(&c.body).duration(Duration::Short).sound(Some(Sound::Default));
    if let Some(hint) = &c.hint {
        t = t.text2(hint);
    }
    for (label, action) in &c.buttons {
        t = t.add_button(label, action);
    }
    let result = t
        .on_activated(move |action| {
            on_click(action);
            Ok(())
        })
        .show();
    if let Err(e) = result {
        eprintln!("peepthis: toast failed: {e}");
    }
}

#[cfg(not(windows))]
pub fn toast(_bundle_id: &str, c: &ToastContent, _on_click: impl Fn(Option<String>) + Send + 'static) {
    println!("peepthis: toast {}: {}", c.title, c.body);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{self, Event};
    use std::sync::Arc;

    const T0: u64 = 1_000_000;
    const AWAY: WidgetView = WidgetView { focused: false, hidden: false, overflowing: false };

    fn session(events: Vec<Event>) -> Session {
        let base = Session::new("a", 1, "demo", "C:/p", "interactive", "cli", T0);
        events.into_iter().fold(base, |s, e| state::apply(s, e, T0))
    }

    fn perm() -> Event {
        Event::Notification { message: "Claude needs your permission to use Bash".into(), idle: false }
    }

    #[test]
    fn notify_first_toast_for_a_session_shows() {
        assert!(should_toast(None, T0, AWAY));
    }

    #[test]
    fn notify_no_toast_within_30s_of_last_for_same_session() {
        assert!(!should_toast(Some(T0), T0 + DEDUP_MS - 1, AWAY));
        assert!(should_toast(Some(T0), T0 + DEDUP_MS, AWAY));
    }

    #[test]
    fn notify_suppressed_when_focused_shown_and_rows_fit() {
        let looking = WidgetView { focused: true, hidden: false, overflowing: false };
        assert!(!should_toast(None, T0, looking));
    }

    #[test]
    fn notify_shown_when_widget_hidden_unfocused_or_scrolling() {
        for view in [
            WidgetView { focused: true, hidden: true, overflowing: false },
            WidgetView { focused: false, hidden: false, overflowing: false },
            WidgetView { focused: true, hidden: false, overflowing: true },
        ] {
            assert!(should_toast(None, T0, view), "{view:?}");
        }
    }

    #[test]
    fn notify_entered_waiting_only_on_transition() {
        let thinking = session(vec![Event::UserPromptSubmit]);
        let waiting = state::apply(thinking.clone(), perm(), T0);
        assert!(entered_waiting(Some(&thinking), &waiting));
        assert!(entered_waiting(None, &waiting));
        let again = state::apply(waiting.clone(), perm(), T0 + 1);
        assert!(!entered_waiting(Some(&waiting), &again));
        let back = state::apply(waiting.clone(), Event::PostToolUse, T0 + 2);
        assert!(!entered_waiting(Some(&waiting), &back));
        let ended = state::apply(waiting.clone(), Event::Ended, T0 + 3);
        assert!(!entered_waiting(Some(&waiting), &ended));
    }

    #[test]
    fn notify_idle_prompt_reminder_never_enters_waiting() {
        let done = session(vec![Event::UserPromptSubmit, Event::Stop]);
        let reminder = state::apply(
            done.clone(),
            Event::Notification { message: "Claude is waiting for your input".into(), idle: true },
            T0 + 1,
        );
        assert!(!entered_waiting(Some(&done), &reminder));
    }

    fn held(id: &str, at: u64, tier: crate::risk::Tier) -> crate::approve::PendingApproval {
        crate::approve::PendingApproval {
            id: id.into(),
            tool: "Read".into(),
            input: serde_json::json!({ "file_path": "C:/p/a.rs" }),
            cwd: "C:/p".into(),
            agent_id: None,
            received_at: at,
            release_at: at + 60_000,
            risk: crate::risk::Verdict { tier, field: None, span: None, reason: "" },
        }
    }

    #[test]
    fn notify_safe_request_gets_allow_and_deny_buttons() {
        use crate::risk::Tier;
        let mut s = session(vec![perm()]);
        s.pending = vec![held("ap-1", T0, Tier::Safe)];
        let c = toast_content(&s);
        assert_eq!(c.title, "demo");
        assert_eq!(c.body, s.task);
        assert_eq!(c.buttons, vec![("Allow", "allow:ap-1".to_string()), ("Deny", "deny:ap-1".to_string())]);
        assert_eq!((c.hint, c.request.as_deref()), (None, Some("ap-1")));
    }

    #[test]
    fn notify_shell_and_destructive_get_no_buttons_and_the_inbox_hint() {
        use crate::risk::Tier;
        for tier in [Tier::Shell, Tier::Destructive] {
            let mut s = session(vec![perm()]);
            s.pending = vec![held("ap-2", T0, tier)];
            let c = toast_content(&s);
            assert!(c.buttons.is_empty(), "{tier:?}");
            assert!(c.hint.as_deref().is_some_and(|h| h.starts_with("Review it in Peep This")), "{tier:?}");
        }
    }

    #[test]
    fn notify_toast_follows_the_newest_request_and_plain_waits_stay_plain() {
        use crate::risk::Tier;
        let mut s = session(vec![perm()]);
        assert_eq!(toast_content(&s), ToastContent { title: "demo".into(), body: s.task.clone(), hint: None, buttons: vec![], request: None });
        s.pending = vec![held("ap-old", T0, Tier::Destructive), held("ap-new", T0 + 5, Tier::Safe)];
        let c = toast_content(&s);
        assert_eq!(c.request.as_deref(), Some("ap-new"));
        assert_eq!(c.buttons.len(), 2);
    }

    #[test]
    fn notify_clicks_route_to_decide_focus_or_nothing() {
        let live = |id: &str| id == "ap-1";
        assert_eq!(click_action(Some("allow:ap-1"), Some("ap-1"), live), Click::Decide { id: "ap-1".into(), allow: true });
        assert_eq!(click_action(Some("deny:ap-1"), Some("ap-1"), live), Click::Decide { id: "ap-1".into(), allow: false });
        assert_eq!(click_action(None, Some("ap-1"), live), Click::Focus, "body still focuses while pending");
        assert_eq!(click_action(None, None, live), Click::Focus, "toast with no request focuses as before");
        let gone = |_: &str| false;
        assert_eq!(click_action(Some("allow:ap-1"), Some("ap-1"), gone), Click::Ignore);
        assert_eq!(click_action(None, Some("ap-1"), gone), Click::Ignore, "resolved toast does nothing");
        assert_eq!(click_action(Some("bogus"), None, live), Click::Ignore);
        assert_eq!(click_action(Some("shred:ap-1"), None, live), Click::Ignore);
    }

    #[test]
    fn notify_notifier_dedups_per_session_and_records_only_shown_toasts() {
        let shown = Arc::new(Mutex::new(Vec::<String>::new()));
        let looking = Arc::new(Mutex::new(false));
        let sink = Arc::clone(&shown);
        let view = Arc::clone(&looking);
        let n = Notifier::new(
            Box::new(move || WidgetView { focused: *view.lock().unwrap(), ..AWAY }),
            Box::new(move |s: &Session| sink.lock().unwrap().push(s.id.clone())),
        );
        let a = session(vec![perm()]);
        let mut b = a.clone();
        b.id = "b".into();

        assert!(n.alert(&a, T0));
        assert!(!n.alert(&a, T0 + 1_000));
        assert!(n.alert(&b, T0 + 1_000), "de-dup is per session");
        *looking.lock().unwrap() = true;
        assert!(!n.alert(&a, T0 + DEDUP_MS));
        *looking.lock().unwrap() = false;
        assert!(n.alert(&a, T0 + DEDUP_MS + 1), "a suppressed alert does not restart the 30s window");
        assert_eq!(*shown.lock().unwrap(), vec!["a", "b", "a"]);
    }
}
