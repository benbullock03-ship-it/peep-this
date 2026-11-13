//! `statusline-original.json`: the user's `statusLine` setting as it was before install (spec §7 step 3).
//!
//! Format: `{"statusLine": <the original settings.json value, verbatim>}`. Uninstall writes it back as is;
//! the wrapper only reads `statusLine.command`. Shared by the installer and the `peepthis-statusline` binary.

use std::path::{Path, PathBuf};

use serde_json::Value;

pub const FILE_NAME: &str = "statusline-original.json";

/// `%APPDATA%\PeepThis`, or `PEEPTHIS_DIR` when set (tests and measurement only).
pub fn dir() -> PathBuf {
    if let Some(d) = std::env::var_os("PEEPTHIS_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(d);
    }
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("PeepThis")
}

pub fn path(dir: &Path) -> PathBuf {
    dir.join(FILE_NAME)
}

pub fn to_json(status_line: &Value) -> String {
    serde_json::json!({ "statusLine": status_line }).to_string()
}

/// The recorded `statusLine` value, if the file parses.
pub fn status_line(raw: &[u8]) -> Option<Value> {
    // Windows PowerShell 5.1 writes a BOM by default.
    let raw = raw.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(raw);
    let v: Value = serde_json::from_slice(raw).ok()?;
    v.get("statusLine").filter(|s| !s.is_null()).cloned()
}

/// The command to run; `None` for no file, bad JSON, or an empty command.
pub fn command(raw: &[u8]) -> Option<String> {
    status_line(raw)?
        .get("command")
        .and_then(Value::as_str)
        .filter(|c| !c.trim().is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_original_value_verbatim() {
        let original = serde_json::json!({
            "type": "command",
            "command": "sh ~/.claude/statusline-command.sh",
            "padding": 0,
            "futureField": [1, 2]
        });
        let raw = to_json(&original);
        assert_eq!(status_line(raw.as_bytes()), Some(original));
        assert_eq!(command(raw.as_bytes()).as_deref(), Some("sh ~/.claude/statusline-command.sh"));
    }

    #[test]
    fn tolerates_a_utf8_bom() {
        let raw = b"\xEF\xBB\xBF{\"statusLine\":{\"command\":\"sh x.sh\"}}";
        assert_eq!(command(raw).as_deref(), Some("sh x.sh"));
    }

    #[test]
    fn missing_or_unusable_command_is_none() {
        for raw in [
            "",
            "{nope",
            "[]",
            "{}",
            r#"{"statusLine":null}"#,
            r#"{"statusLine":{"type":"command"}}"#,
            r#"{"statusLine":{"command":"   "}}"#,
            r#"{"statusLine":{"command":42}}"#,
        ] {
            assert_eq!(command(raw.as_bytes()), None, "{raw}");
        }
    }
}
