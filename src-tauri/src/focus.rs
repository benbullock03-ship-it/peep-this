//! AV-13: PID to top-level window, foregrounding (spec §6.6).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::store::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusError {
    NoWindow,
    Foreground,
}

impl std::fmt::Display for FocusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FocusError::NoWindow => f.write_str("no window found for the session's process tree"),
            FocusError::Foreground => f.write_str("Windows refused to bring the window to the foreground"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcEntry {
    pub parent: u32,
    pub exe: String,
    /// FILETIME ticks; `None` when the process cannot be opened.
    pub started: Option<u64>,
}

/// Processes whose windows are never the session's host (the desktop and taskbar belong to explorer).
const STOP_AT: &[&str] = &["explorer.exe", "services.exe", "wininit.exe", "winlogon.exe", "svchost.exe"];
const MAX_DEPTH: usize = 64;

/// `pid` and its ancestors, nearest first; stops at a cycle, a stop process, or a parent PID that was reused
/// (a parent can never start after its child).
pub fn ancestors(table: &HashMap<u32, ProcEntry>, pid: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut cur = pid;
    while out.len() < MAX_DEPTH && cur != 0 && seen.insert(cur) {
        let Some(e) = table.get(&cur) else { break };
        if STOP_AT.iter().any(|s| e.exe.eq_ignore_ascii_case(s)) {
            break;
        }
        out.push(cur);
        if let (Some(child), Some(parent)) = (e.started, table.get(&e.parent).and_then(|p| p.started)) {
            if parent > child {
                break;
            }
        }
        cur = e.parent;
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub pid: u32,
    pub title: String,
}

/// Windows in z-order; prefers the first whose title mentions `hint` (VS Code/Cursor titles carry the folder name).
pub fn pick_window<'a>(windows: &'a [WindowInfo], pid: u32, hint: Option<&str>) -> Option<&'a WindowInfo> {
    let mut own = windows.iter().filter(|w| w.pid == pid);
    if let Some(h) = hint.map(str::to_lowercase).filter(|h| !h.is_empty()) {
        if let Some(w) = own.clone().find(|w| w.title.to_lowercase().contains(&h)) {
            return Some(w);
        }
    }
    own.next()
}

/// Walks up from `pid`; if nothing in the chain owns a window, falls back to the frontmost Windows Terminal
/// window, because under default-terminal handoff WindowsTerminal.exe is not an ancestor of the shell.
pub fn resolve(
    table: &HashMap<u32, ProcEntry>,
    windows: &[WindowInfo],
    pid: u32,
    hint: Option<&str>,
) -> Option<WindowInfo> {
    for p in ancestors(table, pid) {
        if let Some(w) = pick_window(windows, p, hint) {
            return Some(w.clone());
        }
    }
    windows
        .iter()
        .find(|w| table.get(&w.pid).is_some_and(|e| e.exe.eq_ignore_ascii_case("WindowsTerminal.exe")))
        .cloned()
}

fn folder_hint(cwd: &str) -> Option<&str> {
    cwd.trim_end_matches(['/', '\\']).rsplit(['/', '\\']).next().filter(|s| !s.is_empty())
}

pub fn focus_pid_hint(pid: u32, hint: Option<&str>) -> Result<(), FocusError> {
    let table = sys::with_start_times(sys::process_table(), pid);
    let w = resolve(&table, &sys::top_level_windows(), pid, hint).ok_or(FocusError::NoWindow)?;
    sys::bring_to_front(w.hwnd)
}

/// Whether the window `focus_session` would raise for this session is already the foreground window.
pub fn terminal_is_foreground(pid: u32, cwd: &str) -> bool {
    if !cfg!(windows) || pid == 0 {
        return false;
    }
    let table = sys::with_start_times(sys::process_table(), pid);
    resolve(&table, &sys::top_level_windows(), pid, folder_hint(cwd)).is_some_and(|w| sys::foreground() == Some(w.hwnd))
}

pub fn foreground_window() -> Option<isize> {
    sys::foreground()
}

/// Raises a window remembered earlier; Ok(false) when it has closed since.
pub fn raise(hwnd: isize) -> Result<bool, FocusError> {
    if !sys::exists(hwnd) {
        return Ok(false);
    }
    sys::bring_to_front(hwnd).map(|_| true)
}

/// Ok(false) means no window was found, so the frontend flashes the row.
#[tauri::command]
pub fn focus_session(session_id: String, store: tauri::State<'_, Arc<Store>>) -> Result<bool, String> {
    focus_by_id(&store, &session_id)
}

pub fn focus_by_id(store: &Store, session_id: &str) -> Result<bool, String> {
    if !cfg!(windows) {
        return Ok(false);
    }
    let snap = store.snapshot();
    let Some(s) = snap.sessions.iter().find(|s| s.id == session_id && s.alive) else { return Ok(false) };
    match focus_pid_hint(s.pid, folder_hint(&s.cwd)) {
        Ok(()) => Ok(true),
        Err(FocusError::NoWindow) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(windows)]
mod sys {
    use super::{FocusError, ProcEntry, WindowInfo};
    use crate::source::process::{probe, Proc};
    use std::collections::HashMap;
    use windows_sys::core::BOOL;
    use windows_sys::Win32::Foundation::{CloseHandle, HWND, INVALID_HANDLE_VALUE, LPARAM};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_MENU,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, EnumWindows, GetForegroundWindow, GetWindow, GetWindowLongW, GetWindowTextW,
        GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, SetForegroundWindow, ShowWindow, GWL_EXSTYLE,
        GW_OWNER, SW_RESTORE, WS_EX_TOOLWINDOW,
    };

    pub fn process_table() -> HashMap<u32, ProcEntry> {
        let mut out = HashMap::new();
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snap == INVALID_HANDLE_VALUE {
                return out;
            }
            let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
            let mut ok = Process32FirstW(snap, &mut e);
            while ok != 0 {
                let len = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(e.szExeFile.len());
                out.insert(
                    e.th32ProcessID,
                    ProcEntry { parent: e.th32ParentProcessID, exe: String::from_utf16_lossy(&e.szExeFile[..len]), started: None },
                );
                ok = Process32NextW(snap, &mut e);
            }
            CloseHandle(snap);
        }
        out
    }

    /// Creation times are only needed along the walked chain, so they are filled lazily per lookup.
    pub fn with_start_times(mut table: HashMap<u32, ProcEntry>, pid: u32) -> HashMap<u32, ProcEntry> {
        let mut cur = pid;
        for _ in 0..super::MAX_DEPTH {
            let Some(e) = table.get_mut(&cur) else { break };
            if e.started.is_some() {
                break;
            }
            e.started = match probe(cur) {
                Proc::Alive { started } => started,
                Proc::Dead => None,
            };
            cur = e.parent;
        }
        table
    }

    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let out = &mut *(lparam as *mut Vec<WindowInfo>);
            if IsWindowVisible(hwnd) == 0
                || !GetWindow(hwnd, GW_OWNER).is_null()
                || (GetWindowLongW(hwnd, GWL_EXSTYLE) as u32) & WS_EX_TOOLWINDOW != 0
            {
                return 1;
            }
            let mut buf = [0u16; 512];
            let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if n <= 0 {
                return 1;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            out.push(WindowInfo { hwnd: hwnd as isize, pid, title: String::from_utf16_lossy(&buf[..n as usize]) });
            1
        }
    }

    pub fn top_level_windows() -> Vec<WindowInfo> {
        let mut out: Vec<WindowInfo> = Vec::new();
        unsafe {
            EnumWindows(Some(collect), &mut out as *mut _ as LPARAM);
        }
        out
    }

    pub fn foreground() -> Option<isize> {
        let h = unsafe { GetForegroundWindow() };
        (!h.is_null()).then_some(h as isize)
    }

    pub fn exists(hwnd: isize) -> bool {
        hwnd != 0 && unsafe { IsWindow(hwnd as HWND) } != 0
    }

    fn alt_tap() {
        let key = |flags| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_MENU, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
        };
        let inputs = [key(0), key(KEYEVENTF_KEYUP)];
        unsafe {
            SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
    }

    /// The widget normally holds the foreground (the user just clicked it), so SetForegroundWindow is allowed.
    /// If the lock still refuses, a synthetic Alt tap counts as fresh input and lifts it for the retry.
    pub fn bring_to_front(hwnd: isize) -> Result<(), FocusError> {
        let hwnd = hwnd as HWND;
        unsafe {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            }
            if SetForegroundWindow(hwnd) != 0 && GetForegroundWindow() == hwnd {
                return Ok(());
            }
            alt_tap();
            SetForegroundWindow(hwnd);
            BringWindowToTop(hwnd);
            if GetForegroundWindow() == hwnd { Ok(()) } else { Err(FocusError::Foreground) }
        }
    }
}

#[cfg(not(windows))]
mod sys {
    use super::{FocusError, ProcEntry, WindowInfo};
    use std::collections::HashMap;

    pub fn process_table() -> HashMap<u32, ProcEntry> {
        HashMap::new()
    }
    pub fn with_start_times(table: HashMap<u32, ProcEntry>, _pid: u32) -> HashMap<u32, ProcEntry> {
        table
    }
    pub fn top_level_windows() -> Vec<WindowInfo> {
        Vec::new()
    }
    pub fn bring_to_front(_hwnd: isize) -> Result<(), FocusError> {
        Err(FocusError::NoWindow)
    }
    pub fn foreground() -> Option<isize> {
        None
    }
    pub fn exists(_hwnd: isize) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(parent: u32, exe: &str, started: Option<u64>) -> ProcEntry {
        ProcEntry { parent, exe: exe.into(), started }
    }

    fn w(hwnd: isize, pid: u32, title: &str) -> WindowInfo {
        WindowInfo { hwnd, pid, title: title.into() }
    }

    fn vscode_tree() -> HashMap<u32, ProcEntry> {
        HashMap::from([
            (10, p(20, "claude.exe", Some(500))),
            (20, p(30, "powershell.exe", Some(400))),
            (30, p(40, "Code.exe", Some(300))),
            (40, p(50, "Code.exe", Some(200))),
            (50, p(1, "explorer.exe", Some(100))),
        ])
    }

    #[test]
    fn focus_walk_stops_at_explorer() {
        assert_eq!(ancestors(&vscode_tree(), 10), vec![10, 20, 30, 40]);
    }

    #[test]
    fn focus_walk_guards_cycles() {
        let t = HashMap::from([(1, p(2, "a.exe", None)), (2, p(1, "b.exe", None))]);
        assert_eq!(ancestors(&t, 1), vec![1, 2]);
    }

    #[test]
    fn focus_walk_stops_at_reused_parent_pid() {
        let t = HashMap::from([(10, p(20, "claude.exe", Some(500))), (20, p(30, "Code.exe", Some(900)))]);
        assert_eq!(ancestors(&t, 10), vec![10]);
    }

    #[test]
    fn focus_walk_handles_missing_pid() {
        assert!(ancestors(&vscode_tree(), 999).is_empty());
    }

    #[test]
    fn focus_resolves_nearest_ancestor_window_and_ignores_explorer() {
        let wins = [w(1, 50, "Desktop"), w(2, 40, "main.rs - other - Visual Studio Code")];
        assert_eq!(resolve(&vscode_tree(), &wins, 10, None).unwrap().hwnd, 2);
        assert!(resolve(&vscode_tree(), &[w(1, 50, "Desktop")], 10, None).is_none());
    }

    #[test]
    fn focus_prefers_window_titled_with_project_folder() {
        let wins = [w(1, 40, "a.ts - other - Visual Studio Code"), w(2, 40, "b.rs - AgentView Widget - Visual Studio Code")];
        assert_eq!(resolve(&vscode_tree(), &wins, 10, Some("agentview widget")).unwrap().hwnd, 2);
        assert_eq!(resolve(&vscode_tree(), &wins, 10, Some("nomatch")).unwrap().hwnd, 1);
    }

    #[test]
    fn focus_falls_back_to_windows_terminal_after_handoff() {
        let mut t = HashMap::from([(10, p(20, "claude.exe", None)), (20, p(50, "cmd.exe", None)), (50, p(1, "explorer.exe", None))]);
        t.insert(70, p(50, "WindowsTerminal.exe", None));
        let wins = [w(1, 50, "Desktop"), w(3, 70, "Command Prompt")];
        assert_eq!(resolve(&t, &wins, 10, None).unwrap().hwnd, 3);
    }

    #[test]
    fn focus_folder_hint_takes_last_segment() {
        assert_eq!(folder_hint("F:\\Projects\\agentview widget\\"), Some("agentview widget"));
        assert_eq!(folder_hint("C:/work/demo"), Some("demo"));
        assert_eq!(folder_hint(""), None);
    }

    #[cfg(windows)]
    fn print_resolution(pid: u32, windows: &[WindowInfo]) {
        let table = sys::with_start_times(sys::process_table(), pid);
        let chain: Vec<String> = ancestors(&table, pid).iter().map(|p| format!("{}[{p}]", table[p].exe)).collect();
        let hit = resolve(&table, windows, pid, None);
        println!(
            "pid {pid}: {} => {}",
            chain.join(" <- "),
            hit.map(|w| format!("{}[{}] \"{}\"", table.get(&w.pid).map_or("?", |e| e.exe.as_str()), w.pid, w.title))
                .unwrap_or_else(|| "NO WINDOW".into())
        );
    }

    /// Live check against running sessions (plus `PEEPTHIS_FOCUS_PID` if set); resolves but never foregrounds.
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn focus_live_sessions_resolve() {
        let windows = sys::top_level_windows();
        if let Some(pid) = std::env::var("PEEPTHIS_FOCUS_PID").ok().and_then(|s| s.parse().ok()) {
            print_resolution(pid, &windows);
        }
        let dir = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap()).join(".claude").join("sessions");
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Some(pid) = path.file_stem().and_then(|s| s.to_str()).and_then(|s| s.parse::<u32>().ok()) {
                print_resolution(pid, &windows);
            }
        }
    }
}
