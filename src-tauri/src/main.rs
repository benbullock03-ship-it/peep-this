// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Checked before Tauri starts, so the single-instance plugin never forwards it to a running widget.
    if std::env::args().any(|a| a == peep_this_lib::UNINSTALL_INTEGRATION_ARG) {
        std::process::exit(peep_this_lib::uninstall_integration_cli());
    }
    peep_this_lib::run()
}
