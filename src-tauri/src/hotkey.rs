//! Global hotkeys: the keyboard inbox (AV-24, spec §11.4) and the panel/strip toggle (AV-40,
//! spec §12.6), plus the window focus goes back to after an inbox visit.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::store::{Snapshot, Store};
use crate::{focus, window};

const RESERVED: &[(&str, &str)] = &[("Ctrl+Alt+Delete", "Windows")];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Inbox,
    Toggle,
}

impl Slot {
    const ALL: [Slot; 2] = [Slot::Inbox, Slot::Toggle];

    pub fn default_combo(self) -> &'static str {
        match self {
            Slot::Inbox => "Ctrl+Alt+I",
            Slot::Toggle => "Ctrl+Alt+P",
        }
    }

    fn settings_key(self) -> &'static str {
        match self {
            Slot::Inbox => "inboxHotkey",
            Slot::Toggle => "toggleHotkey",
        }
    }

    fn what(self) -> &'static str {
        match self {
            Slot::Inbox => "the keyboard inbox",
            Slot::Toggle => "the panel/strip toggle",
        }
    }

    fn event(self) -> &'static str {
        match self {
            Slot::Inbox => "hotkey-status",
            Slot::Toggle => "toggle-hotkey-status",
        }
    }

    fn idx(self) -> usize {
        self as usize
    }
}

/// Canonical `Ctrl+Alt+Shift+Win+Key` form, or why the combination cannot be used.
pub fn normalize(raw: &str) -> Result<String, String> {
    let (mut ctrl, mut alt, mut shift, mut win) = (false, false, false, false);
    let mut key: Option<String> = None;
    for tok in raw.split('+').map(str::trim) {
        if tok.is_empty() {
            return Err(format!("\"{raw}\" is not a key combination"));
        }
        let flag = match tok.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => &mut ctrl,
            "alt" => &mut alt,
            "shift" => &mut shift,
            "win" | "super" | "meta" | "cmd" => &mut win,
            _ => {
                if key.is_some() {
                    return Err("Use one key plus modifiers".into());
                }
                key = Some(key_name(tok).ok_or_else(|| format!("\"{tok}\" is not a supported key"))?);
                continue;
            }
        };
        if *flag {
            return Err(format!("{tok} appears twice"));
        }
        *flag = true;
    }
    let Some(key) = key else { return Err("Add a key after the modifiers".into()) };
    if !(ctrl || alt || win) {
        return Err("Include Ctrl, Alt or Win so typing never triggers it".into());
    }
    let mut parts: Vec<&str> = Vec::new();
    for (on, name) in [(ctrl, "Ctrl"), (alt, "Alt"), (shift, "Shift"), (win, "Win")] {
        if on {
            parts.push(name);
        }
    }
    parts.push(&key);
    let combo = parts.join("+");
    if let Some((_, owner)) = RESERVED.iter().find(|(k, _)| *k == combo) {
        return Err(format!("{combo} is reserved for {owner}"));
    }
    Ok(combo)
}

/// The two hotkeys may not share a combination.
fn check_free(combo: &str, other: Slot, other_combo: &str) -> Result<(), String> {
    if combo == other_combo {
        return Err(format!("{combo} is already {}", other.what()));
    }
    Ok(())
}

fn other(slot: Slot) -> Slot {
    match slot {
        Slot::Inbox => Slot::Toggle,
        Slot::Toggle => Slot::Inbox,
    }
}

fn key_name(tok: &str) -> Option<String> {
    let lower = tok.to_ascii_lowercase();
    let bare = lower.strip_prefix("key").filter(|s| s.len() == 1).or_else(|| lower.strip_prefix("digit").filter(|s| s.len() == 1));
    let one = bare.unwrap_or(&lower);
    if one.len() == 1 && one.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Some(one.to_ascii_uppercase());
    }
    if let Some(n) = lower.strip_prefix('f').and_then(|n| n.parse::<u8>().ok()).filter(|n| (1..=24).contains(n)) {
        return Some(format!("F{n}"));
    }
    let named = match lower.as_str() {
        "space" => "Space",
        "home" => "Home",
        "end" => "End",
        "insert" => "Insert",
        "delete" => "Delete",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "up" | "arrowup" => "Up",
        "down" | "arrowdown" => "Down",
        "left" | "arrowleft" => "Left",
        "right" | "arrowright" => "Right",
        _ => return None,
    };
    Some(named.into())
}

fn to_shortcut(combo: &str) -> Result<Shortcut, String> {
    combo.replace("Win", "Super").parse::<Shortcut>().map_err(|e| e.to_string())
}

/// The window the user was in when the hotkey fired; never the widget itself.
#[derive(Debug, Default, PartialEq)]
pub struct Previous(Option<isize>);

impl Previous {
    /// A second press from inside the widget keeps the window remembered by the first.
    pub fn remember(&mut self, foreground: Option<isize>, widget: Option<isize>) {
        if let Some(h) = foreground.filter(|h| *h != 0 && Some(*h) != widget) {
            self.0 = Some(h);
        }
    }

    pub fn take(&mut self) -> Option<isize> {
        self.0.take()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub hotkey: String,
    pub registered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

static STATUS: Mutex<[Option<Status>; 2]> = Mutex::new([None, None]);
static CURRENT: Mutex<[Option<Shortcut>; 2]> = Mutex::new([None, None]);
static PREVIOUS: Mutex<Previous> = Mutex::new(Previous(None));

fn status(slot: Slot) -> Status {
    STATUS.lock().unwrap()[slot.idx()]
        .clone()
        .unwrap_or_else(|| Status { hotkey: slot.default_combo().into(), registered: false, error: None })
}

fn current(slot: Slot) -> Option<Shortcut> {
    CURRENT.lock().unwrap()[slot.idx()]
}

fn set_current(slot: Slot, sc: Option<Shortcut>) {
    CURRENT.lock().unwrap()[slot.idx()] = sc;
}

/// The inbox hotkey as the user would type it, when it is live.
pub fn registered() -> Option<String> {
    let s = status(Slot::Inbox);
    s.registered.then_some(s.hotkey)
}

pub fn pending_count(snap: &Snapshot) -> usize {
    snap.sessions.iter().filter(|s| s.alive).map(|s| s.pending.len()).sum()
}

/// The tray line for a hotkey that is not live, if any.
fn tray_notice(statuses: &[Status]) -> Option<String> {
    statuses.iter().find(|s| !s.registered).map(|s| format!("{} is taken by another app: change it", s.hotkey))
}

fn settings_path() -> PathBuf {
    crate::ingest::server::default_dir().join("settings.json")
}

fn read_settings(path: &Path) -> Value {
    std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null)
}

fn load_saved(path: &Path, slot: Slot) -> Option<String> {
    read_settings(path).get(slot.settings_key()).and_then(Value::as_str).and_then(|s| normalize(s).ok())
}

/// Merges into the settings file the trail switch also lives in.
fn save(path: &Path, slot: Slot, combo: &str) -> std::io::Result<()> {
    let mut v = read_settings(path);
    if !v.is_object() {
        v = Value::Object(Default::default());
    }
    v[slot.settings_key()] = Value::String(combo.into());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&v)?)?;
    std::fs::rename(&tmp, path)
}

pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            match Slot::ALL.into_iter().find(|s| current(*s) == Some(*shortcut)) {
                Some(Slot::Inbox) => pressed(app),
                Some(Slot::Toggle) => window::toggle_shape(app),
                None => {}
            }
        })
        .build()
}

fn register<R: Runtime>(app: &AppHandle<R>, combo: &str) -> Result<Shortcut, String> {
    let sc = to_shortcut(combo)?;
    app.global_shortcut()
        .register(sc)
        .map_err(|e| format!("{combo} could not be registered. Another app is probably using it ({e})."))?;
    Ok(sc)
}

fn set_status<R: Runtime>(app: &AppHandle<R>, slot: Slot, s: Status) {
    STATUS.lock().unwrap()[slot.idx()] = Some(s.clone());
    let _ = app.emit(slot.event(), s);
    crate::tray::hotkey_notice(app, tray_notice(&Slot::ALL.map(status)));
}

pub fn setup<R: Runtime>(app: &AppHandle<R>) {
    let path = settings_path();
    let inbox = load_saved(&path, Slot::Inbox).unwrap_or_else(|| Slot::Inbox.default_combo().into());
    let toggle = load_saved(&path, Slot::Toggle)
        .filter(|c| *c != inbox)
        .unwrap_or_else(|| Slot::Toggle.default_combo().into());
    for (slot, combo) in [(Slot::Inbox, inbox), (Slot::Toggle, toggle)] {
        match register(app, &combo) {
            Ok(sc) => {
                set_current(slot, Some(sc));
                set_status(app, slot, Status { hotkey: combo, registered: true, error: None });
            }
            Err(e) => {
                eprintln!("peepthis: {e}");
                set_status(app, slot, Status { hotkey: combo, registered: false, error: Some(e) });
            }
        }
    }
}

fn pressed<R: Runtime>(app: &AppHandle<R>) {
    let Some(store) = app.try_state::<Arc<Store>>() else { return };
    if pending_count(&store.snapshot()) == 0 {
        let _ = app.emit("inbox-empty", ());
        return;
    }
    PREVIOUS.lock().unwrap().remember(focus::foreground_window(), window::hwnd(app));
    window::summon(app);
    let _ = app.emit("inbox-open", ());
}

/// Swaps a hotkey; on a clash the old one stays registered and the error comes back.
fn rebind(app: &AppHandle, slot: Slot, raw: &str) -> Result<Status, String> {
    let combo = normalize(raw)?;
    check_free(&combo, other(slot), &status(other(slot)).hotkey)?;
    let old = status(slot);
    if old.registered && old.hotkey == combo {
        return Ok(old);
    }
    let old_sc = current(slot);
    if let Some(sc) = old_sc {
        let _ = app.global_shortcut().unregister(sc);
    }
    match register(app, &combo) {
        Ok(sc) => {
            set_current(slot, Some(sc));
            if let Err(e) = save(&settings_path(), slot, &combo) {
                eprintln!("peepthis: saving the {} hotkey failed: {e}", slot.settings_key());
            }
            let s = Status { hotkey: combo, registered: true, error: None };
            set_status(app, slot, s.clone());
            Ok(s)
        }
        Err(e) => {
            if let Some(sc) = old_sc {
                if app.global_shortcut().register(sc).is_err() {
                    set_current(slot, None);
                    set_status(app, slot, Status { registered: false, error: Some(format!("{} was lost while changing it", old.hotkey)), ..old });
                }
            }
            Err(e)
        }
    }
}

#[tauri::command]
pub fn inbox_hotkey() -> Status {
    status(Slot::Inbox)
}

#[tauri::command]
pub fn set_inbox_hotkey(app: AppHandle, hotkey: String) -> Result<Status, String> {
    rebind(&app, Slot::Inbox, &hotkey)
}

#[tauri::command]
pub fn toggle_hotkey() -> Status {
    status(Slot::Toggle)
}

#[tauri::command]
pub fn set_toggle_hotkey(app: AppHandle, hotkey: String) -> Result<Status, String> {
    rebind(&app, Slot::Toggle, &hotkey)
}

/// Ends an inbox visit. `restore` raises the window the hotkey was pressed in; false just forgets it.
#[tauri::command]
pub fn inbox_return(restore: bool) -> bool {
    let Some(h) = PREVIOUS.lock().unwrap().take() else { return false };
    if !restore {
        return false;
    }
    match focus::raise(h) {
        Ok(back) => back,
        Err(e) => {
            eprintln!("peepthis: returning focus failed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: &str = "Ctrl+Alt+I";

    #[test]
    fn hotkey_normalizes_case_order_and_aliases() {
        assert_eq!(normalize("ctrl+alt+i"), Ok("Ctrl+Alt+I".into()));
        assert_eq!(normalize(" Alt + Control + KeyI "), Ok("Ctrl+Alt+I".into()));
        assert_eq!(normalize("shift+super+digit5"), Ok("Shift+Win+5".into()));
        assert_eq!(normalize("Ctrl+Shift+f12"), Ok("Ctrl+Shift+F12".into()));
        assert_eq!(normalize("Win+ArrowUp"), Ok("Win+Up".into()));
        for slot in Slot::ALL {
            assert_eq!(normalize(slot.default_combo()), Ok(slot.default_combo().into()));
        }
    }

    #[test]
    fn hotkey_rejects_unusable_combinations() {
        for bad in ["", "I", "Shift+I", "Ctrl+Alt", "Ctrl++I", "Ctrl+Ctrl+I", "Ctrl+I+J", "Ctrl+Alt+F25", "Ctrl+Alt+Banana", "ctrl+alt+delete"] {
            assert!(normalize(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn hotkey_slots_cannot_share_a_combination() {
        let e = check_free("Ctrl+Alt+P", Slot::Toggle, "Ctrl+Alt+P").unwrap_err();
        assert!(e.contains("panel/strip"), "{e}");
        let e = check_free("Ctrl+Alt+I", Slot::Inbox, "Ctrl+Alt+I").unwrap_err();
        assert!(e.contains("inbox"), "{e}");
        assert!(check_free("Ctrl+Alt+K", Slot::Toggle, "Ctrl+Alt+P").is_ok());
        assert_ne!(Slot::Inbox.default_combo(), Slot::Toggle.default_combo());
    }

    #[test]
    fn hotkey_canonical_forms_parse_as_shortcuts() {
        for combo in [DEFAULT, "Ctrl+Alt+P", "Ctrl+Shift+Win+F5", "Alt+Space", "Ctrl+Alt+PageDown", "Ctrl+9"] {
            assert!(to_shortcut(combo).is_ok(), "{combo}");
        }
    }

    #[test]
    fn hotkey_tray_notice_names_the_first_dead_key() {
        let ok = |k: &str| Status { hotkey: k.into(), registered: true, error: None };
        let dead = |k: &str| Status { hotkey: k.into(), registered: false, error: Some("taken".into()) };
        assert_eq!(tray_notice(&[ok("Ctrl+Alt+I"), ok("Ctrl+Alt+P")]), None);
        let n = tray_notice(&[ok("Ctrl+Alt+I"), dead("Ctrl+Alt+P")]).unwrap();
        assert!(n.starts_with("Ctrl+Alt+P "), "{n}");
    }

    #[test]
    fn hotkey_previous_window_skips_the_widget_and_survives_a_second_press() {
        let mut p = Previous::default();
        p.remember(Some(10), Some(99));
        p.remember(Some(99), Some(99));
        p.remember(None, Some(99));
        assert_eq!(p.take(), Some(10));
        assert_eq!(p.take(), None, "taking clears it");
        p.remember(Some(10), Some(99));
        p.remember(Some(20), Some(99));
        assert_eq!(p.take(), Some(20), "a press from another window replaces it");
        p.remember(Some(0), None);
        assert_eq!(p.take(), None);
    }

    #[test]
    fn hotkey_saved_setting_merges_and_reloads() {
        let dir = std::env::temp_dir().join(format!("peepthis-hotkey-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("settings.json");
        assert_eq!(load_saved(&path, Slot::Inbox), None);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, r#"{"trail":false}"#).unwrap();
        save(&path, Slot::Inbox, "Ctrl+Shift+K").unwrap();
        save(&path, Slot::Toggle, "Ctrl+Alt+O").unwrap();
        assert_eq!(load_saved(&path, Slot::Inbox).as_deref(), Some("Ctrl+Shift+K"));
        assert_eq!(load_saved(&path, Slot::Toggle).as_deref(), Some("Ctrl+Alt+O"));
        assert_eq!(read_settings(&path)["trail"], Value::Bool(false));
        std::fs::write(&path, r#"{"inboxHotkey":"Shift+K"}"#).unwrap();
        assert_eq!(load_saved(&path, Slot::Inbox), None, "an invalid saved key falls back to the default");
        assert_eq!(load_saved(&path, Slot::Toggle), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
