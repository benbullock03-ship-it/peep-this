mod source;
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
    let live: Vec<&str> = snap.sessions.iter().filter(|s| s.alive).map(|s| s.name.as_str()).collect();
    println!(
        "agentview: snapshot {} sessions ({} live): {}",
        snap.sessions.len(),
        live.len(),
        live.join(", ")
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_snapshot])
        .setup(|app| {
            let handle = app.handle().clone();
            let store = Store::new(Box::new(move |snap: &Snapshot| {
                log_snapshot(snap);
                if let Err(e) = handle.emit("sessions", snap) {
                    eprintln!("agentview: emit failed: {e}");
                }
            }));
            app.manage(Arc::clone(&store));

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
