//! Native HTTP hook entries for `~/.claude/settings.json` (spec §6.2). The installer (AV-08) merges these.

use serde_json::{json, Map, Value};

use super::server::TOKEN_HEADER;

/// `SessionStart` is absent: Claude Code rejects `http` hooks on it, and the sessions directory covers it.
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

pub fn hook_url(port: u16, event: &str) -> String {
    format!("http://127.0.0.1:{port}/hook/{event}")
}

/// The `hooks` object for one endpoint. The server answers 204 with no body, which Claude Code treats as "no decision".
pub fn settings_hooks(port: u16, token: &str) -> Value {
    let mut hooks = Map::new();
    for event in EVENTS {
        hooks.insert(
            event.to_string(),
            json!([{ "hooks": [{
                "type": "http",
                "url": hook_url(port, event),
                "timeout": timeout_secs(event),
                "headers": { TOKEN_HEADER: token },
            }]}]),
        );
    }
    Value::Object(hooks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_http_entry_per_event_with_token_header() {
        let v = settings_hooks(47821, "tok");
        let obj = v.as_object().unwrap();
        assert_eq!(obj.len(), EVENTS.len());
        assert!(!obj.contains_key("SessionStart"));
        let h = &v["PermissionRequest"][0]["hooks"][0];
        assert_eq!(h["type"], "http");
        assert_eq!(h["url"], "http://127.0.0.1:47821/hook/PermissionRequest");
        assert_eq!(h["timeout"], 75);
        assert_eq!(h["headers"]["X-PeepThis-Token"], "tok");
        assert_eq!(v["Stop"][0]["hooks"][0]["timeout"], 1);
    }
}
