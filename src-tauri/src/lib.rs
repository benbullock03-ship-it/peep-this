mod approve;
mod cover;
mod focus;
mod hotkey;
mod ingest;
mod install;
mod motion;
mod notify;
mod risk;
mod snap;
mod source;
pub mod statusline_original;
mod tray;
mod window;
#[allow(dead_code)]
mod state;
mod store;
mod ticker;
mod trail;

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
    let live: Vec<String> = snap
        .sessions
        .iter()
        .filter(|s| s.alive)
        .map(|s| {
            let agents: Vec<String> = s
                .agents
                .iter()
                .map(|a| format!("{}={:?} tools={} result={:?}", a.agent_type, a.state, a.tool_count, a.result))
                .collect();
            let shells: Vec<String> = s.shells.iter().map(shell_log).collect();
            let mut line = format!("{}={:?}", s.name, s.state);
            if !agents.is_empty() {
                line.push_str(&format!(" [{}]", agents.join("; ")));
            }
            if !shells.is_empty() {
                line.push_str(&format!(" {{shells: {}}}", shells.join("; ")));
            }
            line
        })
        .collect();
    println!(
        "peepthis: snapshot {} sessions ({} live): {}",
        snap.sessions.len(),
        live.len(),
        live.join(", ")
    );
}

/// `npm run dev=Serving :5173`, `npm run build=Failed exit=1 last="error TS2304"`.
fn shell_log(sh: &state::Shell) -> String {
    let cmd: String = sh.command.chars().take(40).collect();
    let mut out = format!("{cmd}={:?}", sh.state);
    if let Some(p) = sh.port {
        out.push_str(&format!(" :{p}"));
    }
    if let Some(c) = sh.exit_code {
        out.push_str(&format!(" exit={c}"));
    }
    if let Some(l) = &sh.last_line {
        out.push_str(&format!(" last={l:?}"));
    }
    out
}

fn is_pending(store: &Store, id: &str) -> bool {
    store.snapshot().sessions.iter().any(|s| s.pending.iter().any(|p| p.id == id))
}

fn toast_clicked(handle: &tauri::AppHandle, store: &Store, session_id: &str, request: Option<&str>, action: Option<String>) {
    match notify::click_action(action.as_deref(), request, |id| is_pending(store, id)) {
        notify::Click::Ignore => {}
        notify::Click::Focus => match focus::focus_by_id(store, session_id) {
            Ok(true) => {}
            Ok(false) => eprintln!("peepthis: toast click found no window for {session_id}"),
            Err(e) => eprintln!("peepthis: toast click focus failed: {e}"),
        },
        notify::Click::Decide { id, allow } => {
            let Some(broker) = handle.try_state::<Arc<approve::Broker>>() else { return };
            if let Err(e) = broker.decide(&id, allow, None, store::now_ms()) {
                eprintln!("peepthis: toast decision failed: {e}");
            }
        }
    }
}

fn wire_alerts(app: &tauri::App, store: &Arc<Store>) {
    let bundle_id = app.config().identifier.clone();
    let handle = app.handle().clone();
    let weak = Arc::downgrade(store);
    let notifier = notify::Notifier::new(
        Box::new(|| notify::WidgetView {
            focused: window::is_focused(),
            hidden: window::is_hidden(),
            overflowing: window::is_overflowing(),
        }),
        Box::new(move |s| {
            let (s, weak, handle, bundle_id) = (s.clone(), weak.clone(), handle.clone(), bundle_id.clone());
            let spawned = std::thread::Builder::new().name("toast".into()).spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(notify::SHOW_DELAY_MS));
                let fresh = weak.upgrade().and_then(|st| st.snapshot().sessions.into_iter().find(|x| x.id == s.id));
                let content = notify::toast_content(fresh.as_ref().unwrap_or(&s));
                let (id, request) = (s.id.clone(), content.request.clone());
                notify::toast(&bundle_id, &content, move |action| {
                    let Some(store) = weak.upgrade() else { return };
                    toast_clicked(&handle, &store, &id, request.as_deref(), action);
                });
            });
            if let Err(e) = spawned {
                eprintln!("peepthis: toast thread failed: {e}");
            }
        }),
    );
    store.set_alert(Box::new(move |s| {
        notifier.alert(s, store::now_ms());
    }));
}

fn wire_trail(app: &tauri::App, store: &Arc<Store>, broker: &approve::Broker) {
    let trail = trail::Trail::open(&ingest::server::default_dir());
    let pruner = Arc::clone(&trail);
    let spawned = std::thread::Builder::new().name("trail-prune".into()).spawn(move || match pruner.prune(store::now_ms()) {
        Ok(0) => {}
        Ok(n) => println!("peepthis: trail pruned {n} entries"),
        Err(e) => eprintln!("peepthis: trail prune failed: {e}"),
    });
    if let Err(e) = spawned {
        eprintln!("peepthis: trail prune thread failed: {e}");
    }
    app.manage(Arc::clone(&trail));
    let (handle, weak) = (app.handle().clone(), Arc::downgrade(store));
    broker.set_recorder(Box::new(move |d| {
        let name = weak
            .upgrade()
            .and_then(|st| st.snapshot().sessions.into_iter().find(|s| s.id == d.session_id).map(|s| s.name))
            .unwrap_or_default();
        let entry = trail::Entry::from_decision(d, &name);
        match trail.record(&entry) {
            Ok(true) => {
                let _ = handle.emit("trail", &entry);
            }
            Ok(false) => {}
            Err(e) => eprintln!("peepthis: trail write failed: {e}"),
        }
    }));
}

struct WidgetEnv;

impl approve::Env for WidgetEnv {
    fn terminal_foreground(&self, pid: u32, cwd: &str) -> bool {
        focus::terminal_is_foreground(pid, cwd)
    }
    fn widget_hidden(&self) -> bool {
        window::is_hidden()
    }
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
        .plugin(hotkey::plugin())
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            window::fit_window,
            window::get_mode,
            window::switch_mode,
            window::begin_switch,
            window::play_region,
            window::end_switch,
            window::uncover,
            focus::focus_session,
            install::install_integration,
            install::uninstall_integration,
            approve::decide_approval,
            trail::trail_log,
            trail::trail_recent,
            trail::trail_enabled,
            trail::set_trail_enabled,
            hotkey::inbox_hotkey,
            hotkey::set_inbox_hotkey,
            hotkey::inbox_return
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
            ticker::spawn(Arc::clone(&store), Box::new(source::claude::shells::probe));
            let broker = approve::Broker::new(Arc::clone(&store), Box::new(WidgetEnv), approve::HOLD_MS);
            wire_trail(app, &store, &broker);
            broker.spawn();
            app.manage(Arc::clone(&broker));
            let endpoint = ingest::server::spawn(Arc::clone(&store), Some(broker), &ingest::server::default_dir())
                .inspect_err(|e| eprintln!("peepthis: event server failed: {e}"))
                .ok();
            let integrator = install::Integrator::new(install::Paths::for_this_machine(), endpoint);
            integrator.refresh(&store);
            app.manage(integrator);
            tray::setup(app)?;
            hotkey::setup(app.handle());

            let sources: Vec<Box<dyn AgentSource>> = vec![Box::new(ClaudeSource::new())];
            source::spawn_all(sources, || {
                let store = Arc::clone(&store);
                Box::new(move |ev| store.ingest(ev, store::now_ms()))
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let (tauri::RunEvent::Exit, Some(b)) = (event, app.try_state::<Arc<approve::Broker>>()) {
                b.release_all(approve::Release::Quit);
            }
        });
}
