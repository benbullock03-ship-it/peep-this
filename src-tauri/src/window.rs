//! AV-11: docking, height fit and position persistence (spec §3.1).
//! All positions are physical pixels in the virtual desktop; sizes the frontend sends are logical.
//! The window is transparent and larger than the card by the hard shadow on its right and bottom
//! (AV-35). Docking and clamping use the card rect, so the shadow may overhang the margin.

use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Monitor, PhysicalPosition, Runtime, WindowEvent};

use crate::snap::{anchored_origin, Anchor};

pub const WIDTH: f64 = 360.0;
pub const MIN_HEIGHT: f64 = 48.0;
pub const MARGIN: f64 = 12.0;
/// Hard shadow offset (logical); matches `.ps` box-shadow and its right/bottom margin.
pub const SHADOW: f64 = 6.0;
pub const MAX_HEIGHT_FRACTION: f64 = 0.7;
const SAVE_DEBOUNCE: Duration = Duration::from_millis(500);
const SHOW_FALLBACK: Duration = Duration::from_secs(2);
const LABEL: &str = "main";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MonitorId {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    fn right(&self) -> i32 {
        self.x + self.w
    }
    fn bottom(&self) -> i32 {
        self.y + self.h
    }
    fn intersects(&self, o: &Rect) -> bool {
        self.x < o.right() && o.x < self.right() && self.y < o.bottom() && o.y < self.bottom()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MonitorInfo {
    pub id: MonitorId,
    pub work: Rect,
    pub scale: f64,
}

/// Outer window position (physical) and the monitor it was on.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Saved {
    pub monitor: MonitorId,
    pub x: i32,
    pub y: i32,
}

/// `window.json`: the position plus, since AV-38, the anchor. Files without one still parse.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Stored {
    #[serde(flatten)]
    at: Saved,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    anchor: Option<Anchor>,
}

fn to_phys(logical: f64, scale: f64) -> i32 {
    (logical * scale).round() as i32
}

/// `w`/`h` are the card's size; the card shares the window's top-left.
fn clamp_into(work: &Rect, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    let x = x.min(work.right() - w).max(work.x);
    let y = y.min(work.bottom() - h).max(work.y);
    (x, y)
}

fn default_dock(m: &MonitorInfo, w: i32, h: i32) -> (i32, i32) {
    let margin = to_phys(MARGIN, m.scale);
    clamp_into(&m.work, m.work.right() - w - margin, m.work.y + margin, w, h)
}

/// Where the window's outer top-left goes. `size` is the logical window size, shadow included.
/// None when there are no monitors. Saved spot wins if its monitor still exists (same name,
/// position, size) and the saved card overlaps that monitor's work area (clamped fully inside);
/// otherwise the primary's default dock.
#[cfg(test)]
pub fn place(
    saved: Option<&Saved>,
    monitors: &[MonitorInfo],
    primary: Option<&MonitorId>,
    size: (f64, f64),
) -> Option<(i32, i32)> {
    place_anchored(saved, None, monitors, primary, size).map(|(at, _)| at)
}

/// `place` plus the anchor (AV-38). Anchored axes go back to their edge whatever size the card
/// starts at; a save without an anchor is `Float`, or the top-right corner if it sits on the dock.
pub fn place_anchored(
    saved: Option<&Saved>,
    anchor: Option<Anchor>,
    monitors: &[MonitorInfo],
    primary: Option<&MonitorId>,
    size: (f64, f64),
) -> Option<((i32, i32), Anchor)> {
    let card = (size.0 - SHADOW, size.1 - SHADOW);
    let phys = |m: &MonitorInfo| (to_phys(card.0, m.scale), to_phys(card.1, m.scale));
    if let Some(s) = saved {
        if let Some(m) = monitors.iter().find(|m| m.id == s.monitor) {
            let (w, h) = phys(m);
            if m.work.intersects(&Rect { x: s.x, y: s.y, w, h }) {
                let on_dock = (s.x, s.y) == default_dock(m, w, h);
                let a = anchor.unwrap_or(if on_dock { Anchor::TopRight } else { Anchor::Float });
                let (x, y) = anchored_origin(a, m.work, to_phys(MARGIN, m.scale), (w, h), (s.x, s.y));
                return Some((clamp_into(&m.work, x, y, w, h), a));
            }
        }
    }
    let m = primary
        .and_then(|p| monitors.iter().find(|m| &m.id == p))
        .or_else(|| monitors.first())?;
    let (w, h) = phys(m);
    Some((default_dock(m, w, h), Anchor::TopRight))
}

/// Logical window height for `content` logical px (shadow included), capped at 70% of the work
/// area, and the outer top (physical) moved up if the card would cross the work area's bottom.
#[cfg(test)]
pub fn fit(content: f64, top: i32, m: &MonitorInfo) -> (f64, i32) {
    fit_anchored(content, top, m, Anchor::Float)
}

/// `fit`, but a bottom-anchored card keeps its bottom on the margin and grows upward.
pub fn fit_anchored(content: f64, top: i32, m: &MonitorInfo, anchor: Anchor) -> (f64, i32) {
    let cap = (MAX_HEIGHT_FRACTION * m.work.h as f64 / m.scale).floor().max(MIN_HEIGHT);
    let h = content.ceil().clamp(MIN_HEIGHT, cap);
    let card = to_phys(h - SHADOW, m.scale);
    let y = if anchor.is_bottom() {
        m.work.bottom() - to_phys(MARGIN, m.scale) - card
    } else if top + card > m.work.bottom() {
        m.work.bottom() - card
    } else {
        top
    };
    (h, y.max(m.work.y))
}

/// Card and shadow rects in window-relative physical px; their union is the hit-test region,
/// so clicks on the two transparent corners beside the shadow reach the window underneath.
pub fn hit_region(w: i32, h: i32, scale: f64) -> [Rect; 2] {
    let s = to_phys(SHADOW, scale);
    [Rect { x: 0, y: 0, w: w - s, h: h - s }, Rect { x: s, y: s, w: w - s, h: h - s }]
}

#[cfg(windows)]
fn set_region(hwnd: isize, rects: [Rect; 2]) {
    use windows_sys::Win32::Graphics::Gdi::{CombineRgn, CreateRectRgn, DeleteObject, SetWindowRgn, RGN_OR};
    unsafe {
        let [a, b] = rects;
        let rgn = CreateRectRgn(a.x, a.y, a.right(), a.bottom());
        let other = CreateRectRgn(b.x, b.y, b.right(), b.bottom());
        CombineRgn(rgn, rgn, other, RGN_OR);
        DeleteObject(other);
        // The system owns `rgn` once it is set.
        SetWindowRgn(hwnd as _, rgn, 1);
    }
}

/// Live magnet (AV-34 answer): the native caption drag sends `WM_MOVING`, and rewriting its rect
/// catches the card while held. Windows builds each proposal from the previous (snapped) rect, so
/// the raw spot is rebuilt from the cursor and the grab offset; only then does pulling past the
/// magnet break free.
#[cfg(windows)]
mod sys {
    use super::*;
    use crate::snap::{snap, MAGNET};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER, WM_ENTERSIZEMOVE,
        WM_EXITSIZEMOVE, WM_MOVING,
    };

    static DRAG_ANCHOR: Mutex<Option<Anchor>> = Mutex::new(None);
    /// Cursor minus window top-left when the drag began.
    static GRAB: Mutex<Option<(i32, i32)>> = Mutex::new(None);

    fn cursor() -> (i32, i32) {
        let mut p = POINT { x: 0, y: 0 };
        unsafe { GetCursorPos(&mut p) };
        (p.x, p.y)
    }

    pub fn set_rect(h: isize, r: Rect) {
        unsafe { SetWindowPos(h as HWND, std::ptr::null_mut(), r.x, r.y, r.w, r.h, SWP_NOZORDER | SWP_NOACTIVATE) };
    }

    fn scale(h: HWND) -> f64 {
        unsafe { GetDpiForWindow(h) as f64 / 96.0 }
    }

    /// The card part of a window rect (the shadow hangs off its right and bottom).
    fn card_of(rc: &RECT, s: i32) -> Rect {
        Rect { x: rc.left, y: rc.top, w: rc.right - rc.left - s, h: rc.bottom - rc.top - s }
    }

    fn work_for(card: Rect) -> Rect {
        let rc = RECT { left: card.x, top: card.y, right: card.right(), bottom: card.bottom() };
        unsafe {
            let mon = MonitorFromRect(&rc, MONITOR_DEFAULTTONEAREST);
            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(mon, &mut mi);
            let w = mi.rcWork;
            Rect { x: w.left, y: w.top, w: w.right - w.left, h: w.bottom - w.top }
        }
    }

    unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM, _id: usize, _data: usize) -> LRESULT {
        match msg {
            WM_ENTERSIZEMOVE => {
                let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                unsafe { GetWindowRect(hwnd, &mut rc) };
                let (cx, cy) = cursor();
                *GRAB.lock().unwrap() = Some((cx - rc.left, cy - rc.top));
            }
            WM_MOVING => {
                let rc = unsafe { &mut *(lp as *mut RECT) };
                if let Some((gx, gy)) = *GRAB.lock().unwrap() {
                    let (cx, cy) = cursor();
                    let (w, h) = (rc.right - rc.left, rc.bottom - rc.top);
                    (rc.left, rc.top) = (cx - gx, cy - gy);
                    (rc.right, rc.bottom) = (rc.left + w, rc.top + h);
                }
                let k = scale(hwnd);
                let card = card_of(rc, to_phys(SHADOW, k));
                let (to, a) = snap(card, work_for(card), to_phys(MARGIN, k), to_phys(MAGNET, k));
                let (dx, dy) = (to.x - card.x, to.y - card.y);
                (rc.left, rc.top, rc.right, rc.bottom) = (rc.left + dx, rc.top + dy, rc.right + dx, rc.bottom + dy);
                *DRAG_ANCHOR.lock().unwrap() = Some(a);
                return 1;
            }
            WM_EXITSIZEMOVE => {
                *GRAB.lock().unwrap() = None;
                if let Some(a) = DRAG_ANCHOR.lock().unwrap().take() {
                    *ANCHOR.lock().unwrap() = a;
                    // Dropped part-way over a taskbar or off the edge: pull the card back inside.
                    let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    unsafe { GetWindowRect(hwnd, &mut rc) };
                    let card = card_of(&rc, to_phys(SHADOW, scale(hwnd)));
                    let (x, y) = clamp_into(&work_for(card), card.x, card.y, card.w, card.h);
                    if (x, y) != (card.x, card.y) {
                        set_rect(hwnd as isize, Rect { x, y, w: rc.right - rc.left, h: rc.bottom - rc.top });
                    }
                    request_save();
                }
            }
            _ => {}
        }
        unsafe { DefSubclassProc(hwnd, msg, wp, lp) }
    }

    pub fn subclass(h: isize) {
        if unsafe { SetWindowSubclass(h as HWND, Some(proc), 0xA38, 0) } == 0 {
            eprintln!("peepthis: snap subclass failed; dragging works without the magnet");
        }
    }
}

fn info(m: &Monitor) -> MonitorInfo {
    let wa = m.work_area();
    MonitorInfo {
        id: MonitorId {
            name: m.name().cloned().unwrap_or_default(),
            x: m.position().x,
            y: m.position().y,
            width: m.size().width,
            height: m.size().height,
        },
        work: Rect { x: wa.position.x, y: wa.position.y, w: wa.size.width as i32, h: wa.size.height as i32 },
        scale: m.scale_factor(),
    }
}

fn state_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("PeepThis").join("window.json"))
}

fn load() -> Option<Stored> {
    let text = std::fs::read_to_string(state_path()?).ok()?;
    serde_json::from_str(&text).ok()
}

fn store(saved: &Stored) -> std::io::Result<()> {
    let Some(path) = state_path() else { return Ok(()) };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(saved)?)?;
    std::fs::rename(tmp, path)
}

fn save_now<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = app.get_webview_window(LABEL) else { return };
    let (Ok(pos), Ok(Some(mon))) = (w.outer_position(), w.current_monitor()) else { return };
    let saved = Stored { at: Saved { monitor: info(&mon).id, x: pos.x, y: pos.y }, anchor: Some(anchor()) };
    if let Err(e) = store(&saved) {
        eprintln!("peepthis: saving window position failed: {e}");
    }
}

static SAVE_TX: OnceLock<Sender<()>> = OnceLock::new();
/// Hidden from the tray; content-height fits must not re-show the window.
static USER_HIDDEN: AtomicBool = AtomicBool::new(false);
static LAST_CONTENT: Mutex<Option<f64>> = Mutex::new(None);
static FOCUSED: AtomicBool = AtomicBool::new(false);
/// Content is taller than the capped window, so the list scrolls.
static OVERFLOWING: AtomicBool = AtomicBool::new(false);
static ANCHOR: Mutex<Anchor> = Mutex::new(Anchor::TopRight);

pub fn anchor() -> Anchor {
    *ANCHOR.lock().unwrap()
}

fn request_save() {
    if let Some(tx) = SAVE_TX.get() {
        let _ = tx.send(());
    }
}

pub fn is_focused() -> bool {
    FOCUSED.load(Ordering::SeqCst)
}

pub fn is_overflowing() -> bool {
    OVERFLOWING.load(Ordering::SeqCst)
}

fn spawn_saver<R: Runtime>(app: AppHandle<R>) {
    let (tx, rx) = mpsc::channel::<()>();
    if SAVE_TX.set(tx).is_err() {
        return;
    }
    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            loop {
                match rx.recv_timeout(SAVE_DEBOUNCE) {
                    Ok(()) => continue,
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }
            save_now(&app);
        }
    });
}

fn apply_fit<R: Runtime>(w: &tauri::Window<R>, content: f64) -> tauri::Result<()> {
    let mon = match w.current_monitor()? {
        Some(m) => m,
        None => match w.primary_monitor()? {
            Some(m) => m,
            None => return Ok(()),
        },
    };
    let pos = w.outer_position()?;
    let m = info(&mon);
    let (h, y) = fit_anchored(content, pos.y, &m, anchor());
    OVERFLOWING.store(content.ceil() > h, Ordering::SeqCst);
    let (pw, ph) = (to_phys(WIDTH, m.scale), to_phys(h, m.scale));
    #[cfg(windows)]
    {
        // One call: size then position separately shows a frame at the wrong spot (AV-34).
        let hwnd = w.hwnd()?.0 as isize;
        sys::set_rect(hwnd, Rect { x: pos.x, y, w: pw, h: ph });
        set_region(hwnd, hit_region(pw, ph, m.scale));
    }
    #[cfg(not(windows))]
    {
        w.set_size(tauri::PhysicalSize::new(pw as u32, ph as u32))?;
        w.set_position(PhysicalPosition::new(pos.x, y))?;
    }
    if !USER_HIDDEN.load(Ordering::SeqCst) && !w.is_visible()? {
        w.show()?;
    }
    Ok(())
}

pub fn is_hidden() -> bool {
    USER_HIDDEN.load(Ordering::SeqCst)
}

/// Shows or hides the widget and tells the frontend, which detaches its peeps while hidden.
pub fn set_hidden<R: Runtime>(app: &AppHandle<R>, hidden: bool) {
    USER_HIDDEN.store(hidden, Ordering::SeqCst);
    let Some(w) = app.get_webview_window(LABEL) else { return };
    let _ = app.emit("widget-hidden", hidden);
    let result = if hidden {
        w.hide()
    } else {
        w.show().and_then(|_| w.set_focus())
    };
    if let Err(e) = result {
        eprintln!("peepthis: show/hide failed: {e}");
    }
}

#[cfg(windows)]
pub fn hwnd<R: Runtime>(app: &AppHandle<R>) -> Option<isize> {
    app.get_webview_window(LABEL)?.hwnd().ok().map(|h| h.0 as isize)
}

#[cfg(not(windows))]
pub fn hwnd<R: Runtime>(_app: &AppHandle<R>) -> Option<isize> {
    None
}

/// Shows the widget and takes the keyboard (AV-24 inbox).
pub fn summon<R: Runtime>(app: &AppHandle<R>) {
    set_hidden(app, false);
    if let Some(h) = hwnd(app) {
        if crate::focus::foreground_window() != Some(h) {
            if let Err(e) = crate::focus::raise(h) {
                eprintln!("peepthis: inbox could not take focus: {e}");
            }
        }
    }
}

/// Positions the main window (saved spot or default dock) before it is shown.
pub fn setup<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let Some(w) = app.get_webview_window(LABEL) else { return Ok(()) };
    let monitors: Vec<MonitorInfo> = w.available_monitors()?.iter().map(info).collect();
    let primary = w.primary_monitor()?.map(|m| info(&m).id);
    let scale = w.scale_factor()?;
    let size = w.outer_size()?.to_logical::<f64>(scale);
    let stored = load();
    let saved = stored.as_ref().map(|s| &s.at);
    let anchor = stored.as_ref().and_then(|s| s.anchor);
    if let Some(((x, y), a)) = place_anchored(saved, anchor, &monitors, primary.as_ref(), (WIDTH, size.height)) {
        *ANCHOR.lock().unwrap() = a;
        w.set_position(PhysicalPosition::new(x, y))?;
    }
    #[cfg(windows)]
    sys::subclass(w.hwnd()?.0 as isize);
    spawn_saver(app.handle().clone());
    // Shown on the first fit_window call; this covers a frontend that never calls it.
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        std::thread::sleep(SHOW_FALLBACK);
        if let Some(w) = handle.get_webview_window(LABEL).filter(|_| !is_hidden()) {
            let _ = w.show();
        }
    });
    Ok(())
}

pub fn on_event<R: Runtime>(w: &tauri::Window<R>, event: &WindowEvent) {
    if w.label() != LABEL {
        return;
    }
    match event {
        WindowEvent::Focused(f) => FOCUSED.store(*f, Ordering::SeqCst),
        WindowEvent::Moved(_) => request_save(),
        WindowEvent::ScaleFactorChanged { .. } => {
            let last = *LAST_CONTENT.lock().unwrap();
            if let Some(h) = last {
                let _ = apply_fit(w, h);
            }
        }
        _ => {}
    }
}

/// Frontend reports its natural content height (CSS px); the window follows it up to the cap.
#[tauri::command]
pub fn fit_window(window: tauri::Window, height: f64) -> Result<(), String> {
    if !height.is_finite() {
        return Err("height must be finite".into());
    }
    *LAST_CONTENT.lock().unwrap() = Some(height);
    apply_fit(&window, height).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(name: &str, x: i32, y: i32, w: u32, h: u32, scale: f64, taskbar: i32) -> MonitorInfo {
        MonitorInfo {
            id: MonitorId { name: name.into(), x, y, width: w, height: h },
            work: Rect { x, y, w: w as i32, h: h as i32 - taskbar },
            scale,
        }
    }

    const SIZE: (f64, f64) = (WIDTH, 200.0);

    #[test]
    fn window_default_dock_respects_scale_and_taskbar() {
        for (scale, tb) in [(1.0, 48), (1.25, 60), (1.5, 72)] {
            let m = mon("D1", 0, 0, 1920, 1080, scale, tb);
            let (x, y) = place(None, &[m.clone()], Some(&m.id), SIZE).unwrap();
            let card_w = to_phys(WIDTH - SHADOW, scale);
            let margin = to_phys(MARGIN, scale);
            assert_eq!((x, y), (1920 - card_w - margin, margin), "scale {scale}");
            assert!(y + to_phys(200.0 - SHADOW, scale) <= 1080 - tb);
        }
    }

    #[test]
    fn window_default_dock_uses_work_area_offset() {
        let mut m = mon("D1", 0, 0, 1920, 1080, 1.0, 0);
        m.work = Rect { x: 0, y: 0, w: 1872, h: 1080 };
        let (x, _) = place(None, &[m.clone()], Some(&m.id), SIZE).unwrap();
        assert_eq!(x, 1872 - 354 - 12);
    }

    #[test]
    fn window_docked_card_keeps_margin_and_shadow_overhangs_it() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let (x, _) = place(None, &[m.clone()], Some(&m.id), SIZE).unwrap();
        let card_right = x + 354;
        let window_right = x + 360;
        assert_eq!(1920 - card_right, 12);
        assert!(window_right > card_right && window_right <= 1920);
    }

    #[test]
    fn window_card_may_touch_work_edge_with_shadow_past_it() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let s = Saved { monitor: m.id.clone(), x: 1900, y: 2000 };
        let off = place(Some(&s), &[m.clone()], Some(&m.id), SIZE);
        // Fully off the work area: default dock, not a clamp.
        assert_eq!(off, place(None, &[m.clone()], Some(&m.id), SIZE));
        let s = Saved { monitor: m.id.clone(), x: 1800, y: 900 };
        let (x, y) = place(Some(&s), &[m.clone()], Some(&m.id), SIZE).unwrap();
        assert_eq!((x + 354, y + 194), (1920, 1040));
    }

    #[test]
    fn window_saved_position_from_before_av35_restores_same_card_spot() {
        // Pre-AV-35 saves stored the window's top-left, which is also the card's top-left.
        let m = mon("D1", 0, 0, 1920, 1080, 1.25, 50);
        let s = Saved { monitor: m.id.clone(), x: 600, y: 300 };
        assert_eq!(place(Some(&s), &[m.clone()], Some(&m.id), SIZE), Some((600, 300)));
    }

    #[test]
    fn window_hit_region_is_card_plus_offset_shadow() {
        let [card, shadow] = hit_region(540, 300, 1.5);
        assert_eq!(card, Rect { x: 0, y: 0, w: 531, h: 291 });
        assert_eq!(shadow, Rect { x: 9, y: 9, w: 531, h: 291 });
        assert_eq!((shadow.right(), shadow.bottom()), (540, 300));
    }

    #[test]
    fn window_default_dock_on_primary_not_first() {
        let left = mon("D2", -2560, 0, 2560, 1440, 1.0, 40);
        let prim = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        let (x, _) = place(None, &[left, prim.clone()], Some(&prim.id), SIZE).unwrap();
        assert!(x > 0);
    }

    #[test]
    fn window_no_primary_falls_back_to_first_and_no_monitors_is_none() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        assert!(place(None, &[m], None, SIZE).is_some());
        assert_eq!(place(None, &[], None, SIZE), None);
    }

    #[test]
    fn window_restores_saved_on_same_monitor() {
        let prim = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let sec = mon("D2", -2560, -200, 2560, 1440, 1.25, 50);
        let saved = Saved { monitor: sec.id.clone(), x: -1800, y: 100 };
        let got = place(Some(&saved), &[prim.clone(), sec], Some(&prim.id), SIZE);
        assert_eq!(got, Some((-1800, 100)));
    }

    #[test]
    fn window_monitor_gone_or_changed_uses_default_dock() {
        let prim = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let dock = place(None, &[prim.clone()], Some(&prim.id), SIZE);
        let gone = Saved { monitor: mon("D2", 1920, 0, 1920, 1080, 1.0, 40).id, x: 2000, y: 100 };
        assert_eq!(place(Some(&gone), &[prim.clone()], Some(&prim.id), SIZE), dock);
        let mut resized = prim.id.clone();
        resized.width = 2560;
        let moved = Saved { monitor: resized, x: 100, y: 100 };
        assert_eq!(place(Some(&moved), &[prim.clone()], Some(&prim.id), SIZE), dock);
    }

    #[test]
    fn window_offscreen_saved_rect_uses_default_dock() {
        let prim = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let dock = place(None, &[prim.clone()], Some(&prim.id), SIZE);
        let off = Saved { monitor: prim.id.clone(), x: 5000, y: 100 };
        assert_eq!(place(Some(&off), &[prim.clone()], Some(&prim.id), SIZE), dock);
        // Saved behind a taskbar that now covers it.
        let under = Saved { monitor: prim.id.clone(), x: 100, y: 1045 };
        assert_eq!(place(Some(&under), &[prim.clone()], Some(&prim.id), SIZE), dock);
    }

    #[test]
    fn window_partially_offscreen_is_clamped_inside_work_area() {
        let prim = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        let s = Saved { monitor: prim.id.clone(), x: 1800, y: 950 };
        let (x, y) = place(Some(&s), &[prim.clone()], Some(&prim.id), SIZE).unwrap();
        assert_eq!(x, 1920 - to_phys(WIDTH - SHADOW, 1.5));
        assert_eq!(y, 1080 - 72 - to_phys(200.0 - SHADOW, 1.5));
    }

    #[test]
    fn window_fit_caps_at_seventy_percent_of_work_area() {
        for scale in [1.0, 1.25, 1.5] {
            let m = mon("D1", 0, 0, 1920, 1080, scale, 48);
            let (h, _) = fit(5000.0, 0, &m);
            assert_eq!(h, (0.7 * 1032.0 / scale).floor(), "scale {scale}");
            assert!(to_phys(h, scale) <= (0.7 * 1032.0) as i32 + 1);
        }
    }

    #[test]
    fn window_fit_follows_content_with_floor() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.25, 48);
        assert_eq!(fit(180.4, 12, &m), (181.0, 12));
        assert_eq!(fit(10.0, 12, &m).0, MIN_HEIGHT);
    }

    #[test]
    fn window_fit_moves_up_to_stay_above_taskbar() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        let (h, y) = fit(400.0, 900, &m);
        assert_eq!(y, 1008 - to_phys(h - SHADOW, 1.5));
        assert!(y >= 0);
    }

    #[test]
    fn window_first_run_is_the_top_right_corner_anchor() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.25, 48);
        let (at, a) = place_anchored(None, None, &[m.clone()], Some(&m.id), SIZE).unwrap();
        assert_eq!(Some(at), place(None, &[m.clone()], Some(&m.id), SIZE));
        assert_eq!(a, Anchor::TopRight);
    }

    #[test]
    fn window_save_without_anchor_is_float_unless_on_the_dock() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let (dx, dy) = place(None, &[m.clone()], Some(&m.id), SIZE).unwrap();
        let on_dock = Saved { monitor: m.id.clone(), x: dx, y: dy };
        assert_eq!(place_anchored(Some(&on_dock), None, &[m.clone()], Some(&m.id), SIZE).unwrap().1, Anchor::TopRight);
        let elsewhere = Saved { monitor: m.id.clone(), x: 600, y: 300 };
        assert_eq!(place_anchored(Some(&elsewhere), None, &[m.clone()], Some(&m.id), SIZE).unwrap(), ((600, 300), Anchor::Float));
    }

    #[test]
    fn window_bottom_anchor_restores_on_its_edge_at_any_start_height() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        // Saved while tall; launched short. The card's bottom still lands on the margin.
        let s = Saved { monitor: m.id.clone(), x: 300, y: 400 };
        let ((x, y), a) = place_anchored(Some(&s), Some(Anchor::BottomLeft), &[m.clone()], Some(&m.id), SIZE).unwrap();
        assert_eq!(a, Anchor::BottomLeft);
        assert_eq!(x, 18);
        assert_eq!(y + to_phys(200.0 - SHADOW, 1.5), 1008 - 18);
    }

    #[test]
    fn window_bottom_anchored_card_grows_upward() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let bottom = |(h, y): (f64, i32)| y + to_phys(h - SHADOW, 1.0);
        let short = fit_anchored(150.0, 500, &m, Anchor::BottomRight);
        let tall = fit_anchored(450.0, short.1, &m, Anchor::BottomRight);
        assert_eq!(bottom(short), 1040 - 12);
        assert_eq!(bottom(tall), 1040 - 12);
        assert!(tall.1 < short.1);
        // Top-anchored keeps its top.
        assert_eq!(fit_anchored(450.0, 12, &m, Anchor::TopRight).1, 12);
    }

    #[test]
    fn window_stored_file_reads_old_format_and_writes_anchor() {
        let old = r#"{"monitor":{"name":"D1","x":0,"y":0,"width":1920,"height":1080},"x":5,"y":6}"#;
        let s: Stored = serde_json::from_str(old).unwrap();
        assert_eq!((s.at.x, s.at.y, s.anchor), (5, 6, None));
        let new = Stored { anchor: Some(Anchor::Bottom), ..s };
        let text = serde_json::to_string(&new).unwrap();
        assert!(text.contains(r#""anchor":"bottom""#), "{text}");
        assert_eq!(serde_json::from_str::<Stored>(&text).unwrap(), new);
    }

    #[test]
    fn window_saved_state_round_trips_as_json() {
        let s = Saved { monitor: mon("\\\\.\\DISPLAY1", 0, 0, 1920, 1080, 1.0, 40).id, x: -3, y: 7 };
        let back: Saved = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back, s);
    }
}
