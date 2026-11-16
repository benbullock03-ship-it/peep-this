//! AV-113: per-machine peep look overrides keyed by the cwd hash, and the toast icons drawn from them.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use tauri::Emitter;

use crate::away::{read_settings, save_setting, settings_path};

const KEY: &str = "peepLooks";
const COLORS: [&str; 5] = ["yellow", "pink", "lavender", "blue", "green"];
const ICON_MAX_BYTES: usize = 64 * 1024;

/// FNV-1a over the normalised cwd as 8 hex digits; `cwdKey` in `src/look.ts` gives the same.
pub fn cwd_key(cwd: &str) -> String {
    let norm = cwd.trim().to_lowercase().replace('\\', "/");
    let norm = norm.trim_end_matches('/');
    let mut h: u32 = 0x811c9dc5;
    for b in norm.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    format!("{h:08x}")
}

fn valid_key(k: &str) -> bool {
    k.len() == 8 && k.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// The fields worth keeping: a palette colour and an accessory id. None when neither is valid.
fn clean(look: &Value) -> Option<Value> {
    let mut out = Map::new();
    if let Some(c) = look.get("color").and_then(Value::as_str).filter(|c| COLORS.contains(c)) {
        out.insert("color".into(), c.into());
    }
    // The accessory list lives in the frontend (unlocks grow it), so only its shape is checked here.
    if let Some(a) = look
        .get("accessory")
        .and_then(Value::as_str)
        .filter(|a| (1..=24).contains(&a.len()) && a.bytes().all(|b| b.is_ascii_lowercase()))
    {
        out.insert("accessory".into(), a.into());
    }
    (!out.is_empty()).then_some(Value::Object(out))
}

fn saved(path: &Path) -> Map<String, Value> {
    match read_settings(path).get(KEY) {
        Some(Value::Object(m)) => m.iter().filter(|(k, _)| valid_key(k)).filter_map(|(k, v)| Some((k.clone(), clean(v)?))).collect(),
        _ => Map::new(),
    }
}

/// Sets or, with None, removes one project's look; returns every look after the change.
fn set(path: &Path, key: &str, look: Option<&Value>) -> Result<Map<String, Value>, String> {
    if !valid_key(key) {
        return Err(format!("not a cwd key: {key}"));
    }
    let mut all = saved(path);
    match look {
        Some(v) => {
            let v = clean(v).ok_or("no valid colour or accessory")?;
            all.insert(key.into(), v);
        }
        None => {
            all.remove(key);
        }
    }
    save_setting(path, KEY, Value::Object(all.clone())).map_err(|e| e.to_string())?;
    Ok(all)
}

#[tauri::command]
pub fn peep_looks() -> Map<String, Value> {
    saved(&settings_path())
}

#[tauri::command]
pub fn set_peep_look(key: String, look: Option<Value>, app: tauri::AppHandle) -> Result<Map<String, Value>, String> {
    let all = set(&settings_path(), &key, look.as_ref())?;
    let _ = app.emit("peep-looks", &all);
    Ok(all)
}

fn icon_dir() -> PathBuf {
    crate::ingest::server::default_dir().join("icons")
}

/// The project's toast icon, if the frontend has drawn one.
pub fn icon_for(cwd: &str) -> Option<PathBuf> {
    let p = icon_dir().join(format!("{}.png", cwd_key(cwd)));
    p.is_file().then_some(p)
}

fn is_png(bytes: &[u8]) -> bool {
    bytes.len() <= ICON_MAX_BYTES && bytes.starts_with(b"\x89PNG\r\n\x1a\n")
}

#[tauri::command]
pub fn set_toast_icon(cwd: String, png: Vec<u8>) -> Result<(), String> {
    if !is_png(&png) {
        return Err("not a small PNG".into());
    }
    let dir = icon_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.png", cwd_key(&cwd)));
    let tmp = path.with_extension("png.tmp");
    std::fs::write(&tmp, &png).and_then(|_| std::fs::rename(&tmp, &path)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_cwd_key_matches_the_frontend_for_every_spelling() {
        for cwd in ["C:\\Users\\Dev\\Projects\\AgentView", "c:/users/dev/projects/agentview/", " C:/Users/Dev/Projects/AgentView "] {
            assert_eq!(cwd_key(cwd), "62cc5ae9", "{cwd}");
        }
        assert_eq!(cwd_key(""), "811c9dc5");
    }

    #[test]
    fn look_clean_keeps_only_palette_colours_and_accessory_ids() {
        let v = serde_json::json!({ "color": "pink", "accessory": "wizard", "extra": 1 });
        assert_eq!(clean(&v), Some(serde_json::json!({ "color": "pink", "accessory": "wizard" })));
        assert_eq!(clean(&serde_json::json!({ "color": "#fff" })), None);
        assert_eq!(clean(&serde_json::json!({ "color": "red", "accessory": "packed" })), Some(serde_json::json!({ "accessory": "packed" })));
        assert_eq!(clean(&serde_json::json!({ "accessory": "../x" })), None);
    }

    #[test]
    fn look_set_round_trips_and_reset_removes_the_entry() {
        let dir = std::env::temp_dir().join(format!("peepthis-look-{}", std::process::id()));
        let path = dir.join("settings.json");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, r#"{"alertSound":true,"peepLooks":{"BAD":{"color":"pink"},"00000001":{"color":"nope"}}}"#).unwrap();
        assert!(saved(&path).is_empty(), "bad keys and empty looks are dropped on read");
        let all = set(&path, "62cc5ae9", Some(&serde_json::json!({ "color": "green" }))).unwrap();
        assert_eq!(all["62cc5ae9"], serde_json::json!({ "color": "green" }));
        assert_eq!(saved(&path), all, "survives a re-read, as after a restart");
        assert_eq!(read_settings(&path)["alertSound"], Value::from(true), "other settings are kept");
        assert!(set(&path, "62cc5ae9", None).unwrap().is_empty());
        assert!(set(&path, "../../x", None).is_err());
        assert!(set(&path, "62cc5ae9", Some(&serde_json::json!({ "color": "red" }))).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn look_icon_must_be_a_small_png() {
        assert!(is_png(b"\x89PNG\r\n\x1a\nrest"));
        assert!(!is_png(b"GIF89a"));
        let mut big = b"\x89PNG\r\n\x1a\n".to_vec();
        big.resize(ICON_MAX_BYTES + 1, 0);
        assert!(!is_png(&big));
    }
}
