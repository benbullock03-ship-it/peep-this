//! The learned list (AV-110): tips shown, Peep skills run and patterns a Goose health report cited, kept in settings.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::away::{read_settings, save_setting, settings_path};
use crate::store::Store;
use crate::tip::Tip;

const KEY: &str = "learned";
/// AV-109's once-ever tips, saved before this list existed.
const TIPS_SEEN_KEY: &str = "tipsSeen";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Learned {
    pub tips: BTreeSet<String>,
    pub skills: BTreeSet<String>,
    pub patterns: BTreeSet<String>,
    pub rhythm: Rhythm,
}

/// The rhythm counter (AV-137): local only, never sent anywhere.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Rhythm {
    pub packups: Count,
    pub tickets: Count,
    pub approvals: Count,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Count {
    pub count: u32,
    /// Epoch ms of the first one.
    pub first: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Habit {
    PackUp,
    TicketClosed,
    Approval,
}

impl Rhythm {
    pub fn bump(&mut self, h: Habit, now: u64) {
        let c = match h {
            Habit::PackUp => &mut self.packups,
            Habit::TicketClosed => &mut self.tickets,
            Habit::Approval => &mut self.approvals,
        };
        c.count += 1;
        c.first.get_or_insert(now);
    }
}

pub fn tip_id(t: Tip) -> String {
    serde_json::to_value(t).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default()
}

/// Adds every id not yet in `set`; true if any was new.
pub fn add<I: IntoIterator<Item = String>>(set: &mut BTreeSet<String>, ids: I) -> bool {
    ids.into_iter().fold(false, |new, id| set.insert(id) | new)
}

pub fn read(path: &Path) -> Learned {
    let v = read_settings(path);
    let mut l: Learned = v.get(KEY).and_then(|x| serde_json::from_value(x.clone()).ok()).unwrap_or_default();
    if let Some(Value::Array(seen)) = v.get(TIPS_SEEN_KEY) {
        add(&mut l.tips, seen.iter().filter_map(|t| t.as_str().map(str::to_owned)));
    }
    l
}

pub fn save(path: &Path, l: &Learned) {
    if let Err(e) = save_setting(path, KEY, serde_json::to_value(l).unwrap_or(Value::Null)) {
        eprintln!("peepthis: saving the learned list failed: {e}");
    }
}

/// Pattern ids on the cards of `<root>/.peep/graph/candidates.json`, written by `/peep:goose health` (schema: ducks references/report.md §2).
pub fn cited_patterns(root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(root.join(".peep").join("graph").join("candidates.json")) else { return vec![] };
    let Ok(v) = serde_json::from_str::<Value>(&text) else { return vec![] };
    v.get("cards")
        .and_then(Value::as_array)
        .map(|cards| cards.iter().filter_map(|c| c.get("pattern")?.as_str()).filter(|p| valid_id(p)).map(str::to_owned).collect())
        .unwrap_or_default()
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 40 && id.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
}

pub fn load(store: &Store) {
    let path = settings_path();
    store.set_learned(read(&path), Some(path));
}

#[tauri::command]
pub fn learned(store: tauri::State<'_, Arc<Store>>) -> Learned {
    store.learned_with(|roots| roots.iter().flat_map(|r| cited_patterns(Path::new(r))).collect())
}

/// A lesson's source link, opened in the browser; only https, so the page cannot launch anything else.
#[tauri::command]
pub fn open_source(url: String) -> Result<(), String> {
    if !is_web_link(&url) {
        return Err("not an https link".into());
    }
    crate::tray::open_url(&url);
    Ok(())
}

fn is_web_link(url: &str) -> bool {
    url.starts_with("https://") && !url.chars().any(|c| c.is_whitespace() || c == '"')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("peepthis-learned-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn learned_round_trips_and_folds_in_the_old_tips_seen() {
        let dir = tmp("settings");
        let path = dir.join("settings.json");
        save_setting(&path, TIPS_SEEN_KEY, serde_json::json!(["helper"])).unwrap();
        let mut l = read(&path);
        assert_eq!(l.tips, BTreeSet::from(["helper".to_string()]));
        assert!(add(&mut l.skills, ["ducks".to_string()]));
        assert!(!add(&mut l.skills, ["ducks".to_string()]));
        add(&mut l.tips, [tip_id(Tip::Crowded)]);
        save(&path, &l);
        let back = read(&path);
        assert_eq!(back.tips, BTreeSet::from(["crowded".to_string(), "helper".to_string()]));
        assert_eq!(back.skills, BTreeSet::from(["ducks".to_string()]));
        assert_eq!(read_settings(&path)[TIPS_SEEN_KEY], serde_json::json!(["helper"]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rhythm_round_trips_with_first_use_dates() {
        let dir = tmp("rhythm");
        let path = dir.join("settings.json");
        let mut l = read(&path);
        l.rhythm.bump(Habit::Approval, 5);
        l.rhythm.bump(Habit::Approval, 9);
        l.rhythm.bump(Habit::PackUp, 7);
        save(&path, &l);
        let r = read(&path).rhythm;
        assert_eq!(r.approvals, Count { count: 2, first: Some(5) });
        assert_eq!(r.packups, Count { count: 1, first: Some(7) });
        assert_eq!(r.tickets, Count::default());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn learned_opens_only_https_links() {
        assert!(is_web_link("https://arxiv.org/abs/2307.03172"));
        assert!(!is_web_link("file:///C:/Windows/System32/calc.exe"));
        assert!(!is_web_link("C:\\Windows\\notepad.exe"));
        assert!(!is_web_link("https://x.org/a b"));
    }

    #[test]
    fn learned_reads_cited_patterns_from_the_health_report() {
        let dir = tmp("report");
        assert!(cited_patterns(&dir).is_empty());
        std::fs::create_dir_all(dir.join(".peep/graph")).unwrap();
        let cards = r#"{"top":0,"cards":[{"pattern":"god-file"},{"pattern":"leaky-seam"},{"pattern":"../x"},{"name":"no pattern"}]}"#;
        std::fs::write(dir.join(".peep/graph/candidates.json"), cards).unwrap();
        assert_eq!(cited_patterns(&dir), vec!["god-file", "leaky-seam"]);
        let _ = std::fs::remove_dir_all(dir);
    }
}
