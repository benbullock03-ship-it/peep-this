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

/// Shows the toast on its own thread; `on_click` runs when the user clicks it while the widget is running.
#[cfg(windows)]
pub fn toast(bundle_id: &str, title: String, body: String, on_click: impl Fn() + Send + 'static) {
    use tauri_winrt_notification::{Duration, Sound, Toast};
    // Dev builds have no installed Start-menu shortcut carrying the bundle AUMID, so Windows would drop their toasts.
    let app_id = if cfg!(debug_assertions) { Toast::POWERSHELL_APP_ID.to_string() } else { bundle_id.to_string() };
    let spawned = std::thread::Builder::new().name("toast".into()).spawn(move || {
        let result = Toast::new(&app_id)
            .title(&title)
            .text1(&body)
            .duration(Duration::Short)
            .sound(Some(Sound::Default))
            .on_activated(move |_| {
                on_click();
                Ok(())
            })
            .show();
        if let Err(e) = result {
            eprintln!("peepthis: toast failed: {e}");
        }
    });
    if let Err(e) = spawned {
        eprintln!("peepthis: toast thread failed: {e}");
    }
}

#[cfg(not(windows))]
pub fn toast(_bundle_id: &str, title: String, body: String, _on_click: impl Fn() + Send + 'static) {
    println!("peepthis: toast {title}: {body}");
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
