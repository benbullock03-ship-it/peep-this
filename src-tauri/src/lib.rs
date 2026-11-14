mod focus;
mod ingest;
mod install;
mod notify;
mod risk;
mod source;
pub mod statusline_original;
mod tray;
mod window;
#[allow(dead_code)]
mod state;
mod store;
mod ticker;

use std::sync::Arc;

use tauri::{Emitter, Manager};

use source::claude::ClaudeSource;
use source::AgentSource;
use store::{Snapshot, Store};

#[tauri::command]
fn get_snapshot(store: tauri::State<'_, Arc<Store>>) -> Snapshot {
    store.snapshot()
}

fn log_snapshot(snap: &Snapshot) {
    let live: Vec<String> =
        snap.sessions.iter().filter(|s| s.alive).map(|s| format!("{}={:?}", s.name, s.state)).collect();
    println!(
        "peepthis: snapshot {} sessions ({} live): {}",
        snap.sessions.len(),
        live.len(),
        live.join(", ")
    );
}

fn wire_alerts(app: &tauri::App, store: &Arc<Store>) {
    let bundle_id = app.config().identifier.clone();
    let focus_store = Arc::downgrade(store);
    let notifier = notify::Notifier::new(
        Box::new(|| notify::WidgetView {
            focused: window::is_focused(),
            hidden: window::is_hidden(),
            overflowing: window::is_overflowing(),
        }),
        Box::new(move |s| {
            let (id, store) = (s.id.clone(), focus_store.clone());
            notify::toast(&bundle_id, s.name.clone(), s.task.clone(), move || {
                let Some(store) = store.upgrade() else { return };
                match focus::focus_by_id(&store, &id) {
                    Ok(true) => {}
                    Ok(false) => eprintln!("peepthis: toast click found no window for {id}"),
                    Err(e) => eprintln!("peepthis: toast click focus failed: {e}"),
                }
            });
        }),
    );
    store.set_alert(Box::new(move |s| {
        notifier.alert(s, store::now_ms());
    }));
}

pub const UNINSTALL_INTEGRATION_ARG: &str = "--uninstall-integration";

/// Headless: the NSIS uninstaller runs this before removing files. Exit code 0 on success.
pub fn uninstall_integration_cli() -> i32 {
    match install::remove_for_app_uninstall(&install::Paths::for_this_machine()) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("peepthis: {e}");
            1
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first: a second launch exits here and shows the running widget.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| window::set_hidden(app, false)))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            window::fit_window,
            focus::focus_session,
            install::install_integration,
            install::uninstall_integration
        ])
        .on_window_event(|w, e| window::on_event(w, e))
        .setup(|app| {
            window::setup(app)?;
            let handle = app.handle().clone();
            let store = Store::new(Box::new(move |snap: &Snapshot| {
                log_snapshot(snap);
                if let Err(e) = handle.emit("sessions", snap) {
                    eprintln!("peepthis: emit failed: {e}");
                }
            }));
            app.manage(Arc::clone(&store));
            wire_alerts(app, &store);
            ticker::spawn(Arc::clone(&store));
            let endpoint = ingest::server::spawn(Arc::clone(&store), &ingest::server::default_dir())
                .inspect_err(|e| eprintln!("peepthis: event server failed: {e}"))
                .ok();
            let integrator = install::Integrator::new(install::Paths::for_this_machine(), endpoint);
            integrator.refresh(&store);
            app.manage(integrator);
            tray::setup(app)?;

            let sources: Vec<Box<dyn AgentSource>> = vec![Box::new(ClaudeSource::new())];
            source::spawn_all(sources, || {
                let store = Arc::clone(&store);
                Box::new(move |ev| store.ingest(ev, store::now_ms()))
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
