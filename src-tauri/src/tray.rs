//! Tray icon and menu (spec §3.1), plus start-at-login.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_autostart::ManagerExt;

use crate::install::Integrator;
use crate::store::Store;
use crate::window;

const ICON: &[u8] = include_bytes!("../icons/tray.png");
/// Written once start-at-login has been switched on by default, so a user's later "off" sticks.
const AUTOSTART_MARKER: &str = "autostart-default-applied";
/// README.md carries the same link.
const SPONSOR_URL: &str = "https://github.com/sponsors/benbullock03-ship-it";

#[derive(Clone, Serialize)]
struct TrayResult {
    action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn marker_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("PeepThis").join(AUTOSTART_MARKER))
}

/// Start at login defaults to on. Debug builds skip the default so `tauri dev` never registers a dev exe.
fn apply_autostart_default<R: Runtime>(app: &AppHandle<R>) {
    if cfg!(debug_assertions) {
        return;
    }
    let Some(marker) = marker_path() else { return };
    if marker.exists() {
        return;
    }
    if let Err(e) = app.autolaunch().enable() {
        eprintln!("peepthis: enabling start at login failed: {e}");
        return;
    }
    if let Some(dir) = marker.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&marker, b"");
}

fn toggle<R: Runtime>(app: &AppHandle<R>) {
    window::set_hidden(app, !window::is_hidden());
}

#[cfg(windows)]
pub(crate) fn open_url(url: &str) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
    let (verb, file) = (wide("open"), wide(url));
    let code = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) };
    // ShellExecuteW reports success as a value above 32.
    if code as usize <= 32 {
        eprintln!("peepthis: opening {url} failed ({})", code as usize);
    }
}

#[cfg(not(windows))]
pub(crate) fn open_url(url: &str) {
    eprintln!("peepthis: opening links is Windows-only; visit {url}");
}

fn run_integration<R: Runtime>(app: &AppHandle<R>, action: &'static str) {
    let app = app.clone();
    // Install copies files and rewrites settings; keep it off the event loop.
    std::thread::spawn(move || {
        let it = app.state::<Integrator>();
        let store = app.state::<Arc<Store>>();
        let result = if action == "install" { it.install(&store) } else { it.uninstall(&store) };
        let error = result.err();
        if error.is_some() {
            window::set_mode(&app, window::Mode::Panel);
        }
        let _ = app.emit("integration-result", TrayResult { action, error });
    });
}

/// A hotkey another app holds (AV-40): a line at the top of the tray menu that opens settings.
struct HotkeyNotice<R: Runtime> {
    menu: Menu<R>,
    item: MenuItem<R>,
    sep: PredefinedMenuItem<R>,
    shown: Mutex<bool>,
}

/// Shows `text` at the top of the tray menu, or removes the line when None.
pub fn hotkey_notice<R: Runtime>(app: &AppHandle<R>, text: Option<String>) {
    let Some(n) = app.try_state::<HotkeyNotice<R>>() else { return };
    let mut shown = n.shown.lock().unwrap();
    let want = text.is_some();
    let result = match text {
        Some(t) => n.item.set_text(t).and_then(|_| if *shown { Ok(()) } else { n.menu.insert_items(&[&n.item, &n.sep], 0) }),
        None if *shown => n.menu.remove(&n.item).and_then(|_| n.menu.remove(&n.sep)),
        None => Ok(()),
    };
    match result {
        Ok(()) => *shown = want,
        Err(e) => eprintln!("peepthis: tray hotkey notice failed: {e}"),
    }
}

/// The major.minor of `version` when no fixture folder covers it (AV-135).
fn untested(version: &str, fixtures: &str) -> Option<String> {
    fn major_minor(v: &str) -> Option<String> {
        let mut parts = v.trim().split('.');
        let (major, minor) = (parts.next()?, parts.next()?);
        let num = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
        (num(major) && num(minor)).then(|| format!("{major}.{minor}"))
    }
    let mm = major_minor(version)?;
    let known = fixtures.split(',').any(|f| major_minor(f).as_deref() == Some(mm.as_str()));
    (!known).then_some(mm)
}

/// A quiet line at the top of the tray menu, added once for the first untested version seen.
struct VersionNotice<R: Runtime> {
    menu: Menu<R>,
    shown: Mutex<bool>,
}

pub fn version_notice<R: Runtime>(app: &AppHandle<R>, version: &str) {
    let Some(n) = app.try_state::<VersionNotice<R>>() else { return };
    let mut shown = n.shown.lock().unwrap();
    if *shown {
        return;
    }
    let Some(mm) = untested(version, env!("PEEP_FIXTURE_VERSIONS")) else { return };
    let result = MenuItem::with_id(app, "untested-version", format!("untested Claude Code version {mm}"), false, None::<&str>)
        .and_then(|item| PredefinedMenuItem::separator(app).and_then(|sep| n.menu.insert_items(&[&item, &sep], 0)));
    match result {
        Ok(()) => *shown = true,
        Err(e) => eprintln!("peepthis: tray version notice failed: {e}"),
    }
}

pub fn setup<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let handle = app.handle();
    apply_autostart_default(handle);

    let show_hide = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<&str>)?;
    let strip = MenuItem::with_id(app, "strip", "Collapse to strip", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Decision log and settings", true, None::<&str>)?;
    let learned = MenuItem::with_id(app, "learned", "What Peep taught you", true, None::<&str>)?;
    let install = MenuItem::with_id(app, "install", "Install integration", true, None::<&str>)?;
    let uninstall = MenuItem::with_id(app, "uninstall", "Uninstall integration", true, None::<&str>)?;
    let autostart_on = handle.autolaunch().is_enabled().unwrap_or(false);
    let autostart = CheckMenuItem::with_id(app, "autostart", "Start at login", true, autostart_on, None::<&str>)?;
    let version = MenuItem::with_id(app, "version", format!("Peep This {}", env!("CARGO_PKG_VERSION")), false, None::<&str>)?;
    let sponsor = MenuItem::with_id(app, "sponsor", "Sponsor Peep This", true, None::<&str>)?;
    let about = Submenu::with_items(app, "About", true, &[&version, &sponsor])?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep = || PredefinedMenuItem::separator(app);
    let menu = Menu::with_items(app, &[&show_hide, &strip, &settings, &learned, &sep()?, &install, &uninstall, &sep()?, &autostart, &about, &sep()?, &quit])?;
    app.manage(HotkeyNotice {
        menu: menu.clone(),
        item: MenuItem::with_id(app, "hotkey-clash", "", true, None::<&str>)?,
        sep: sep()?,
        shown: Mutex::new(false),
    });
    app.manage(VersionNotice { menu: menu.clone(), shown: Mutex::new(false) });

    let autostart_item = autostart.clone();
    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(ICON)?)
        .tooltip("Peep This")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => toggle(app),
            "strip" => window::set_mode(app, window::Mode::Strip),
            "settings" | "hotkey-clash" => {
                window::set_mode(app, window::Mode::Panel);
                let _ = app.emit("open-settings", ());
            }
            "learned" => {
                window::set_mode(app, window::Mode::Panel);
                let _ = app.emit("open-learned", ());
            }
            "install" => run_integration(app, "install"),
            "uninstall" => run_integration(app, "uninstall"),
            "autostart" => {
                let launcher = app.autolaunch();
                let want = !launcher.is_enabled().unwrap_or(false);
                let result = if want { launcher.enable() } else { launcher.disable() };
                if let Err(e) = result {
                    eprintln!("peepthis: start at login change failed: {e}");
                }
                let _ = autostart_item.set_checked(launcher.is_enabled().unwrap_or(false));
            }
            "sponsor" => open_url(SPONSOR_URL),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sponsor_url_is_https() {
        // ShellExecute runs whatever it is given; only a web link may reach it.
        assert!(SPONSOR_URL.starts_with("https://"));
    }

    #[test]
    fn untested_compares_major_minor() {
        let known = "2.1.325,2.1.326";
        assert_eq!(untested("2.2.0", known).as_deref(), Some("2.2"));
        assert_eq!(untested("3.1.1", known).as_deref(), Some("3.1"));
        assert_eq!(untested("2.1.400", known), None);
        assert_eq!(untested("", known), None);
        assert_eq!(untested("dev", known), None);
        assert_eq!(untested("2.2.0", "").as_deref(), Some("2.2"));
    }

    #[test]
    fn fixture_versions_include_captured_folders() {
        assert!(env!("PEEP_FIXTURE_VERSIONS").split(',').any(|v| v.starts_with("2.1.")));
        assert!(!env!("PEEP_FIXTURE_VERSIONS").contains("legacy"));
    }
}
