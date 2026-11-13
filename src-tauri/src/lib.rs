mod focus;
mod ingest;
mod source;
mod window;
#[allow(dead_code)]
mod state;
mod store;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_snapshot, window::fit_window, focus::focus_session])
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
            if let Err(e) = ingest::server::spawn(Arc::clone(&store), &ingest::server::default_dir()) {
                eprintln!("peepthis: event server failed: {e}");
            }

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
