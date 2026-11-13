//! Tray icon and menu (spec §3.1), plus start-at-login.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_autostart::ManagerExt;

use crate::install::Integrator;
use crate::store::Store;
use crate::window;

const ICON: &[u8] = include_bytes!("../icons/tray.png");
/// Written once start-at-login has been switched on by default, so a user's later "off" sticks.
const AUTOSTART_MARKER: &str = "autostart-default-applied";

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

fn run_integration<R: Runtime>(app: &AppHandle<R>, action: &'static str) {
    let app = app.clone();
    // Install copies files and rewrites settings; keep it off the event loop.
    std::thread::spawn(move || {
        let it = app.state::<Integrator>();
        let store = app.state::<Arc<Store>>();
        let result = if action == "install" { it.install(&store) } else { it.uninstall(&store) };
        let error = result.err();
        if error.is_some() {
            window::set_hidden(&app, false);
        }
        let _ = app.emit("integration-result", TrayResult { action, error });
    });
}

pub fn setup<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let handle = app.handle();
    apply_autostart_default(handle);

    let show_hide = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<&str>)?;
    let install = MenuItem::with_id(app, "install", "Install integration", true, None::<&str>)?;
    let uninstall = MenuItem::with_id(app, "uninstall", "Uninstall integration", true, None::<&str>)?;
    let autostart_on = handle.autolaunch().is_enabled().unwrap_or(false);
    let autostart = CheckMenuItem::with_id(app, "autostart", "Start at login", true, autostart_on, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep = || PredefinedMenuItem::separator(app);
    let menu = Menu::with_items(app, &[&show_hide, &sep()?, &install, &uninstall, &sep()?, &autostart, &sep()?, &quit])?;

    let autostart_item = autostart.clone();
    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(ICON)?)
        .tooltip("Peep This")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => toggle(app),
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
