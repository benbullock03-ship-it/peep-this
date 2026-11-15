//! Command hook entries for `~/.claude/settings.json` (spec §6.2). The installer (AV-08) merges these.
//! Each runs the relay (`<relay> hook <Event>`), which posts to the widget and stays silent when it is closed (AV-73).

use serde_json::{json, Map, Value};

/// `SessionStart` is absent: the sessions directory covers it.
pub const EVENTS: [&str; 11] = [
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionRequest",
    "Notification",
    "Stop",
    "StopFailure",
    "SubagentStart",
    "SubagentStop",
    "SessionEnd",
];

/// Seconds. Claude Code's default is 600; a hung widget must not stall a turn.
pub const TIMEOUT_SECS: u64 = 1;
/// The broker answers within its hold window; this is only the backstop if the widget hangs.
pub const PERMISSION_TIMEOUT_SECS: u64 = crate::approve::HOLD_MS / 1000 + 15;

pub fn timeout_secs(event: &str) -> u64 {
    if event == "PermissionRequest" { PERMISSION_TIMEOUT_SECS } else { TIMEOUT_SECS }
}

/// `relay` is the already-quoted relay command.
pub fn hook_command(relay: &str, event: &str) -> String {
    format!("{relay} hook {event}")
}

/// The `hooks` object. The relay prints nothing unless the widget answers with a body (a permission decision).
pub fn settings_hooks(relay: &str) -> Value {
    let mut hooks = Map::new();
    for event in EVENTS {
        hooks.insert(
            event.to_string(),
            json!([{ "hooks": [{
                "type": "command",
                "command": hook_command(relay, event),
                "timeout": timeout_secs(event),
            }]}]),
        );
    }
    Value::Object(hooks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_command_entry_per_event_without_token() {
        let v = settings_hooks("\"C:/a b/peepthis-statusline.exe\"");
        let obj = v.as_object().unwrap();
        assert_eq!(obj.len(), EVENTS.len());
        assert!(!obj.contains_key("SessionStart"));
        let h = &v["PermissionRequest"][0]["hooks"][0];
        assert_eq!(h["type"], "command");
        assert_eq!(h["command"], "\"C:/a b/peepthis-statusline.exe\" hook PermissionRequest");
        assert_eq!(h["timeout"], 75);
        assert!(h.get("headers").is_none());
        assert_eq!(v["Stop"][0]["hooks"][0]["timeout"], 1);
    }
}
