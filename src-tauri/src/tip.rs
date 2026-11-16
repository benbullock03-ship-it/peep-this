//! Signal tips (AV-109): `state::apply` raises `Session::signal`; the store shows each tip once.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::away::{read_settings, save_setting, settings_path};
use crate::store::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tip {
    /// Once per session; the rest are once ever.
    Crowded,
    Misses,
    Cold,
    Helper,
    Compacted,
}

/// Absolute, because a % misleads on 1M windows (reflective pass §4).
pub const CROWDED_TOKENS: u64 = 120_000;
/// Fallback when the payload has no token count: 120k of a 200k window.
pub const CROWDED_PCT: f64 = 60.0;
pub const MISSES: u32 = 2;
/// Subscription cache lifetime, used until a statusline reports `prompt_cache.ttl`.
pub const CACHE_TTL_DEFAULT_MS: u64 = 60 * 60 * 1000;
const SEEN_KEY: &str = "tipsSeen";

pub fn crowded(tokens: Option<u64>, pct: Option<f64>) -> bool {
    match tokens {
        Some(t) => t >= CROWDED_TOKENS,
        None => pct.is_some_and(|p| p >= CROWDED_PCT),
    }
}

pub fn ttl_ms(ttl: &str) -> Option<u64> {
    match ttl {
        "5m" => Some(5 * 60 * 1000),
        "1h" => Some(60 * 60 * 1000),
        _ => None,
    }
}

pub fn once_ever(tip: Tip) -> bool {
    tip != Tip::Crowded
}

fn saved(path: &Path) -> HashSet<Tip> {
    read_settings(path)
        .get(SEEN_KEY)
        .and_then(|v| serde_json::from_value::<Vec<Tip>>(v.clone()).ok())
        .map(|v| v.into_iter().collect())
        .unwrap_or_default()
}

pub fn save(path: &Path, seen: &HashSet<Tip>) {
    let mut list: Vec<Tip> = seen.iter().copied().collect();
    list.sort_by_key(|t| *t as u8);
    if let Err(e) = save_setting(path, SEEN_KEY, serde_json::to_value(list).unwrap_or(Value::Null)) {
        eprintln!("peepthis: saving seen tips failed: {e}");
    }
}

pub fn load(store: &Store) {
    let path = settings_path();
    store.set_tips_seen(saved(&path), Some(path));
}

#[tauri::command]
pub fn dismiss_tip(session_id: String, store: tauri::State<'_, Arc<Store>>) {
    store.dismiss_tip(&session_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crowded_keys_on_tokens_before_percent() {
        assert!(crowded(Some(120_000), Some(12.0)), "120k of a 1M window is 12%");
        assert!(!crowded(Some(119_999), Some(90.0)));
        assert!(crowded(None, Some(60.0)));
        assert!(!crowded(None, Some(59.0)));
        assert!(!crowded(None, None));
    }

    #[test]
    fn seen_tips_round_trip_through_settings_keeping_other_keys() {
        let dir = std::env::temp_dir().join(format!("peepthis-tips-{}", std::process::id()));
        let path = dir.join("settings.json");
        save_setting(&path, "trail", Value::from(true)).unwrap();
        save(&path, &[Tip::Helper, Tip::Cold].into_iter().collect());
        assert_eq!(saved(&path), [Tip::Helper, Tip::Cold].into_iter().collect());
        assert_eq!(read_settings(&path)["trail"], Value::from(true));
        let _ = std::fs::remove_dir_all(dir);
    }
}
