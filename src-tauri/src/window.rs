//! AV-11: docking, height fit and position persistence (spec §3.1).
//! All positions are physical pixels in the virtual desktop; sizes the frontend sends are logical.
//! The window is transparent and larger than the card by the hard shadow on its right and bottom
//! (AV-35). Docking and clamping use the card rect, so the shadow may overhang the margin.

use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
/// Strip plate thickness (logical); matches `.ps-strip` in styles.css.
pub const STRIP: f64 = 62.0;
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

/// `window.json`: the position plus the anchor (AV-38) and mode (AV-39). Files without them still parse.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Stored {
    #[serde(flatten)]
    at: Saved,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    anchor: Option<Anchor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mode: Option<Mode>,
}

/// Spec §12.5. `Hidden` keeps the last visible shape underneath.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Panel,
    Strip,
    Hidden,
}

/// What the window is shaped like: the panel, or the strip as a column or a row.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Panel,
    Column,
    Row,
}

impl Layout {
    /// The strip is a row only on the top and bottom edges; corners and floating are columns.
    pub fn of(strip: bool, anchor: Anchor) -> Layout {
        match (strip, anchor) {
            (false, _) => Layout::Panel,
            (true, Anchor::Top | Anchor::Bottom) => Layout::Row,
            (true, _) => Layout::Column,
        }
    }

    /// Logical window size before the first fit; only the fixed axis matters.
    fn initial_size(self, length: f64) -> (f64, f64) {
        match self {
            Layout::Panel => (WIDTH, length),
            Layout::Column => (STRIP + SHADOW, length),
            Layout::Row => (length, STRIP + SHADOW),
        }
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct ModeView {
    pub mode: Mode,
    pub layout: Layout,
    pub anchor: Anchor,
    /// The panel is showing as a hover peek (AV-41); `mode` stays `strip`.
    pub peek: bool,
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

/// Logical panel height for `content` logical px (shadow included), capped at 70% of the work area.
pub fn panel_height(content: f64, m: &MonitorInfo) -> f64 {
    let cap = (MAX_HEIGHT_FRACTION * m.work.h as f64 / m.scale).floor().max(MIN_HEIGHT);
    content.ceil().clamp(MIN_HEIGHT, cap)
}

/// The window rect (physical, shadow included) for `layout` holding `content` logical px along its
/// free axis, and whether the content overflows it. Anchored edges stay on their margin, so a
/// bottom- or right-anchored card grows up or left; free axes keep `base` and are clamped inside.
pub fn target(layout: Layout, content: f64, base: (i32, i32), m: &MonitorInfo, anchor: Anchor) -> (Rect, bool) {
    let thick = STRIP + SHADOW;
    let strip_len = |work: i32| content.ceil().clamp(thick, (work as f64 / m.scale).floor().max(thick));
    let (lw, lh) = match layout {
        Layout::Panel => (WIDTH, panel_height(content, m)),
        Layout::Column => (thick, strip_len(m.work.h)),
        Layout::Row => (strip_len(m.work.w), thick),
    };
    let (cw, ch) = (to_phys(lw - SHADOW, m.scale), to_phys(lh - SHADOW, m.scale));
    let (x, y) = anchored_origin(anchor, m.work, to_phys(MARGIN, m.scale), (cw, ch), base);
    let (x, y) = clamp_into(&m.work, x, y, cw, ch);
    let overflow = layout == Layout::Panel && content.ceil() > lh;
    (Rect { x, y, w: to_phys(lw, m.scale), h: to_phys(lh, m.scale) }, overflow)
}

/// Card and shadow rects in window-relative physical px; their union is the hit-test region,
/// so clicks on the two transparent corners beside the shadow reach the window underneath.
pub fn hit_region(w: i32, h: i32, scale: f64) -> [Rect; 2] {
    let s = to_phys(SHADOW, scale);
    [Rect { x: 0, y: 0, w: w - s, h: h - s }, Rect { x: s, y: s, w: w - s, h: h - s }]
}

#[cfg(windows)]
fn set_region(hwnd: isize, rects: &[Rect]) {
    use windows_sys::Win32::Graphics::Gdi::{CombineRgn, CreateRectRgn, DeleteObject, SetWindowRgn, RGN_OR};
    unsafe {
        let rgn = CreateRectRgn(0, 0, 0, 0);
        for r in rects {
            let part = CreateRectRgn(r.x, r.y, r.right(), r.bottom());
            CombineRgn(rgn, rgn, part, RGN_OR);
            DeleteObject(part);
        }
        // The system owns `rgn` once it is set.
        SetWindowRgn(hwnd as _, rgn, 1);
    }
}

fn union(a: Rect, b: Rect) -> Rect {
    let (x, y) = (a.x.min(b.x), a.y.min(b.y));
    Rect { x, y, w: a.right().max(b.right()) - x, h: a.bottom().max(b.bottom()) - y }
}

/// What the frontend animates between (AV-39): the union window's size and both shapes' window
/// rects inside it, in CSS px.
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct FoldView {
    pub win: [f64; 2],
    pub from: [f64; 4],
    pub to: [f64; 4],
    /// The resize is under a frozen picture of `from`: the frontend draws its beacon at `from`'s
    /// bottom-right corner and holds the animation until `uncovered`.
    pub covered: bool,
}

/// The window rect that holds both shapes, and what the frontend needs to animate inside it.
pub fn fold_view(from: Rect, to: Rect, scale: f64) -> (Rect, FoldView) {
    let u = union(from, to);
    let css = |v: i32| v as f64 / scale;
    let inside = |r: Rect| [css(r.x - u.x), css(r.y - u.y), css(r.w), css(r.h)];
    (u, FoldView { win: [css(u.w), css(u.h)], from: inside(from), to: inside(to), covered: false })
}

/// A CSS-px rect as physical px, rounded outward so the region never clips what it covers.
pub fn cover(r: [f64; 4], scale: f64) -> Rect {
    let (x, y) = ((r[0] * scale).floor() as i32, (r[1] * scale).floor() as i32);
    Rect { x, y, w: ((r[0] + r[2]) * scale).ceil() as i32 - x, h: ((r[1] + r[3]) * scale).ceil() as i32 - y }
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

    fn contains(r: &Rect, (x, y): (i32, i32)) -> bool {
        x >= r.x && x < r.right() && y >= r.y && y < r.bottom()
    }

    /// The cursor is on the window's rect, or on `extra`.
    pub fn cursor_over(h: isize, extra: Option<Rect>) -> bool {
        let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if unsafe { GetWindowRect(h as HWND, &mut rc) } == 0 {
            return false;
        }
        let win = Rect { x: rc.left, y: rc.top, w: rc.right - rc.left, h: rc.bottom - rc.top };
        let at = cursor();
        contains(&win, at) || extra.is_some_and(|r| contains(&r, at))
    }

    pub fn button_down() -> bool {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON};
        [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON].iter().any(|&k| unsafe { GetAsyncKeyState(k as i32) } as u16 & 0x8000 != 0)
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
                DRAGGING.store(true, Ordering::SeqCst);
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
                DRAGGING.store(false, Ordering::SeqCst);
                *GRAB.lock().unwrap() = None;
                if let Some(a) = DRAG_ANCHOR.lock().unwrap().take() {
                    *ANCHOR.lock().unwrap() = a;
                    *TARGET.lock().unwrap() = None;
                    *HOME.lock().unwrap() = None;
                    // Dropped part-way over a taskbar or off the edge: pull the card back inside.
                    let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    unsafe { GetWindowRect(hwnd, &mut rc) };
                    let card = card_of(&rc, to_phys(SHADOW, scale(hwnd)));
                    let (x, y) = clamp_into(&work_for(card), card.x, card.y, card.w, card.h);
                    if (x, y) != (card.x, card.y) {
                        set_rect(hwnd as isize, Rect { x, y, w: rc.right - rc.left, h: rc.bottom - rc.top });
                    }
                    request_save();
                    // The strip turns between column and row with the anchor.
                    if let Some(f) = ANCHOR_MOVED.get() {
                        f();
                    }
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
    // A peek is not a move: the strip's own spot is what gets saved.
    let home = if is_peeking() { *HOME.lock().unwrap() } else { None };
    let (x, y) = home.unwrap_or((pos.x, pos.y));
    let saved = Stored { at: Saved { monitor: info(&mon).id, x, y }, anchor: Some(anchor()), mode: Some(mode()) };
    if let Err(e) = store(&saved) {
        eprintln!("peepthis: saving window position failed: {e}");
    }
}

static SAVE_TX: OnceLock<Sender<()>> = OnceLock::new();
/// Hidden from the tray; content-height fits must not re-show the window.
static USER_HIDDEN: AtomicBool = AtomicBool::new(false);
static LAST_CONTENT: Mutex<Option<(Layout, f64)>> = Mutex::new(None);
/// The visible shape; `USER_HIDDEN` sits on top of it. The app launches in the strip.
static STRIP_SHAPE: AtomicBool = AtomicBool::new(true);
/// A strip/panel switch in flight: the window sits on the union of both shapes while the frontend
/// animates the card inside it, then drops to `to`.
struct Fold {
    id: u64,
    hwnd: isize,
    /// The union rect the window sits on meanwhile.
    on: Rect,
    to: Rect,
    scale: f64,
}
static FOLD: Mutex<Option<Fold>> = Mutex::new(None);
static FOLD_ID: AtomicU64 = AtomicU64::new(0);
/// A frontend that never ends its switch is ended for it.
const FOLD_TIMEOUT: Duration = Duration::from_millis(1500);
/// Where the strip was before it unfolded; folding goes back there even if the panel was clamped.
static HOME: Mutex<Option<(i32, i32)>> = Mutex::new(None);
/// The rect last asked for.
static TARGET: Mutex<Option<Rect>> = Mutex::new(None);
static ANCHOR_MOVED: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
static FOCUSED: AtomicBool = AtomicBool::new(false);
/// Content is taller than the capped window, so the list scrolls.
static OVERFLOWING: AtomicBool = AtomicBool::new(false);
static ANCHOR: Mutex<Anchor> = Mutex::new(Anchor::TopRight);
/// The panel shape is up as a hover peek over the strip (AV-41); the mode is still `strip`.
static PEEK: AtomicBool = AtomicBool::new(false);
/// The strip's window rect when the peek opened; the cursor there still counts as on the peek.
static PEEK_STRIP: Mutex<Option<Rect>> = Mutex::new(None);
/// A native caption drag is under way.
static DRAGGING: AtomicBool = AtomicBool::new(false);

pub fn anchor() -> Anchor {
    *ANCHOR.lock().unwrap()
}

pub fn is_peeking() -> bool {
    PEEK.load(Ordering::SeqCst)
}

pub fn mode() -> Mode {
    match (USER_HIDDEN.load(Ordering::SeqCst), STRIP_SHAPE.load(Ordering::SeqCst) || is_peeking()) {
        (true, _) => Mode::Hidden,
        (false, true) => Mode::Strip,
        (false, false) => Mode::Panel,
    }
}

fn layout() -> Layout {
    Layout::of(STRIP_SHAPE.load(Ordering::SeqCst), anchor())
}

pub fn mode_view() -> ModeView {
    ModeView { mode: mode(), layout: layout(), anchor: anchor(), peek: is_peeking() }
}

fn emit_mode<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.emit("mode", mode_view());
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

struct Plan {
    now: Rect,
    to: Rect,
    scale: f64,
}

/// Where the window is and where `layout` holding `content` puts it. None without a monitor.
fn plan<R: Runtime>(w: &tauri::Window<R>, layout: Layout, content: f64) -> tauri::Result<Option<Plan>> {
    let mon = match w.current_monitor()? {
        Some(m) => m,
        None => match w.primary_monitor()? {
            Some(m) => m,
            None => return Ok(None),
        },
    };
    let (pos, size) = (w.outer_position()?, w.outer_size()?);
    let now = Rect { x: pos.x, y: pos.y, w: size.width as i32, h: size.height as i32 };
    let m = info(&mon);
    let home = if layout == Layout::Panel { None } else { HOME.lock().unwrap().take() };
    let from = TARGET.lock().unwrap().unwrap_or(now);
    let (to, overflow) = target(layout, content, home.unwrap_or((from.x, from.y)), &m, anchor());
    // In the strip no row is in view, so alerts treat it like an overflowing list.
    OVERFLOWING.store(overflow || layout != Layout::Panel, Ordering::SeqCst);
    *TARGET.lock().unwrap() = Some(to);
    Ok(Some(Plan { now, to, scale: m.scale }))
}

/// Ends the switch in flight (only `id`'s, if given): one `SetWindowPos` to the final rect.
/// With `painted`, a resize is hidden under a frozen picture of the final rect until the frontend's
/// beacon shows there; `painted` then runs. True when that picture went up.
fn finish_fold(id: Option<u64>, painted: Option<Painted>) -> bool {
    let fold = {
        let mut slot = FOLD.lock().unwrap();
        if id.is_some() && slot.as_ref().map(|f| f.id) != id {
            return false;
        }
        slot.take()
    };
    // The lock is released first: SetWindowPos from a worker waits on the window's thread.
    let Some(f) = fold else { return false };
    crate::motion::cancel();
    let covered = cfg!(windows) && f.on != f.to && painted.is_some();
    if let Some(then) = painted.filter(|_| covered) {
        crate::cover::show_until_painted(f.to, then);
    }
    #[cfg(windows)]
    {
        sys::set_rect(f.hwnd, f.to);
        set_region(f.hwnd, &hit_region(f.to.w, f.to.h, f.scale));
    }
    #[cfg(not(windows))]
    let _ = (f.hwnd, f.scale);
    covered
}

type Painted = Box<dyn FnOnce() + Send>;

/// Tells the frontend the frozen picture is off and its own pixels are on screen.
fn painted_signal<R: Runtime>(app: &AppHandle<R>) -> Painted {
    let app = app.clone();
    Box::new(move || {
        let _ = app.emit("uncovered", ());
    })
}

fn apply_fit<R: Runtime>(w: &tauri::Window<R>, layout: Layout, content: f64) -> tauri::Result<()> {
    finish_fold(None, None);
    // A measurement of the shape just left; the new one follows.
    if layout != self::layout() {
        return Ok(());
    }
    let Some(p) = plan(w, layout, content)? else { return Ok(()) };
    #[cfg(windows)]
    {
        let hwnd = w.hwnd()?.0 as isize;
        // One call: size then position separately shows a frame at the wrong spot (AV-34).
        sys::set_rect(hwnd, p.to);
        set_region(hwnd, &hit_region(p.to.w, p.to.h, p.scale));
    }
    #[cfg(not(windows))]
    {
        let _ = (p.now, p.scale);
        w.set_size(tauri::PhysicalSize::new(p.to.w as u32, p.to.h as u32))?;
        w.set_position(PhysicalPosition::new(p.to.x, p.to.y))?;
    }
    if !USER_HIDDEN.load(Ordering::SeqCst) && !w.is_visible()? {
        w.show()?;
    }
    Ok(())
}

/// Starts a strip/panel switch: sizes the window once to the union of the shape it has and the
/// one `layout` needs, with the hit region left on the card still showing. None when there is
/// nothing to animate (hidden, reduced motion); the frontend then swaps and fits as usual.
fn start_fold<R: Runtime>(w: &tauri::Window<R>, layout: Layout, content: f64) -> tauri::Result<Option<FoldView>> {
    finish_fold(None, None);
    if cfg!(not(windows)) || layout != self::layout() || crate::motion::reduced_motion() || !w.is_visible()? {
        return Ok(None);
    }
    let Some(p) = plan(w, layout, content)? else { return Ok(None) };
    if p.now == p.to {
        return Ok(None);
    }
    #[cfg(windows)]
    {
        let hwnd = w.hwnd()?.0 as isize;
        let (u, mut view) = fold_view(p.now, p.to, p.scale);
        let (dx, dy) = (p.now.x - u.x, p.now.y - u.y);
        view.covered = u != p.now;
        if view.covered {
            crate::cover::show_until_painted(p.now, painted_signal(w.app_handle()));
        }
        // Region first. Set after the resize, the old strip-shaped region would sit at the union's
        // top-left for a moment and show the strip there, at the panel's corner.
        set_region(hwnd, &hit_region(p.now.w, p.now.h, p.scale).map(|r| Rect { x: r.x + dx, y: r.y + dy, ..r }));
        sys::set_rect(hwnd, u);
        let id = FOLD_ID.fetch_add(1, Ordering::SeqCst) + 1;
        *FOLD.lock().unwrap() = Some(Fold { id, hwnd, on: u, to: p.to, scale: p.scale });
        std::thread::spawn(move || {
            std::thread::sleep(FOLD_TIMEOUT);
            finish_fold(Some(id), None);
        });
        return Ok(Some(view));
    }
    #[allow(unreachable_code)]
    Ok(None)
}

pub fn is_hidden() -> bool {
    USER_HIDDEN.load(Ordering::SeqCst)
}

/// Shows or hides the widget and tells the frontend, which detaches its peeps while hidden.
pub fn set_hidden<R: Runtime>(app: &AppHandle<R>, hidden: bool) {
    apply_hidden(app, hidden, true);
}

fn apply_hidden<R: Runtime>(app: &AppHandle<R>, hidden: bool, activate: bool) {
    finish_fold(None, None);
    // Hidden from a peek: the strip is what comes back.
    if hidden && PEEK.swap(false, Ordering::SeqCst) {
        set_shape(app, true);
        *PEEK_STRIP.lock().unwrap() = None;
    }
    USER_HIDDEN.store(hidden, Ordering::SeqCst);
    let Some(w) = app.get_webview_window(LABEL) else { return };
    let _ = app.emit("widget-hidden", hidden);
    emit_mode(app);
    request_save();
    let result = if hidden {
        w.hide()
    } else if activate {
        w.show().and_then(|_| w.set_focus())
    } else {
        show_inactive(&w)
    };
    if let Err(e) = result {
        eprintln!("peepthis: show/hide failed: {e}");
    }
}

/// Shows the window without taking the keyboard from the app in front (AV-40).
fn show_inactive<R: Runtime>(w: &tauri::WebviewWindow<R>) -> tauri::Result<()> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNOACTIVATE};
        unsafe { ShowWindow(w.hwnd()?.0 as _, SW_SHOWNOACTIVATE) };
    }
    // Brings tao's visible flag along; ShowWindow(SW_SHOW) on a window already shown does not activate it.
    w.show()
}

/// The hotkey's target: panel, or a peek, folds to the strip; strip, or hidden, opens the panel.
pub fn toggled(from: Mode, peek: bool) -> Mode {
    match from {
        _ if peek => Mode::Strip,
        Mode::Panel => Mode::Strip,
        Mode::Strip | Mode::Hidden => Mode::Panel,
    }
}

/// The panel/strip hotkey (AV-40). Never takes focus, even when it brings the widget back.
pub fn toggle_shape<R: Runtime>(app: &AppHandle<R>) {
    switch_to(app, toggled(mode(), is_peeking()), false);
}

/// Switches between strip and panel (shown if it was hidden), or hides. On the `mode` event the
/// frontend asks for `begin_switch`, animates the card, then `end_switch`.
pub fn set_mode<R: Runtime>(app: &AppHandle<R>, to: Mode) {
    switch_to(app, to, true);
}

/// Puts the window in the strip's or the panel's shape. Unfolding notes the strip's spot in HOME,
/// which the next strip fit takes, so folding goes back there.
fn set_shape<R: Runtime>(app: &AppHandle<R>, strip: bool) {
    if STRIP_SHAPE.swap(strip, Ordering::SeqCst) == strip {
        return;
    }
    // HOME is read from the strip's own spot, not from a union still on screen.
    finish_fold(None, None);
    if !strip {
        let at = app.get_webview_window(LABEL).and_then(|w| w.outer_position().ok());
        *HOME.lock().unwrap() = at.map(|p| (p.x, p.y));
    }
}

/// Any explicit switch ends a peek: to `panel` pins it, to `strip` folds it.
fn switch_to<R: Runtime>(app: &AppHandle<R>, to: Mode, activate: bool) {
    if to == Mode::Hidden {
        return set_hidden(app, true);
    }
    if PEEK.swap(false, Ordering::SeqCst) {
        *PEEK_STRIP.lock().unwrap() = None;
    }
    set_shape(app, to == Mode::Strip);
    if is_hidden() {
        apply_hidden(app, false, activate);
    } else {
        emit_mode(app);
        request_save();
    }
}

/// Unfolds the panel over the strip without changing the mode or taking focus (AV-41).
fn start_peek<R: Runtime>(app: &AppHandle<R>) {
    if is_hidden() || !STRIP_SHAPE.load(Ordering::SeqCst) || PEEK.swap(true, Ordering::SeqCst) {
        return;
    }
    let rect = app.get_webview_window(LABEL).and_then(|w| {
        let (p, s) = (w.outer_position().ok()?, w.outer_size().ok()?);
        Some(Rect { x: p.x, y: p.y, w: s.width as i32, h: s.height as i32 })
    });
    *PEEK_STRIP.lock().unwrap() = rect;
    set_shape(app, false);
    emit_mode(app);
}

fn end_peek<R: Runtime>(app: &AppHandle<R>) {
    if !PEEK.swap(false, Ordering::SeqCst) {
        return;
    }
    set_shape(app, true);
    *PEEK_STRIP.lock().unwrap() = None;
    emit_mode(app);
}

/// Polls the cursor against the strip and a peek, and opens or folds the peek (AV-41).
#[cfg(windows)]
fn spawn_peek<R: Runtime>(app: AppHandle<R>, hwnd: isize) {
    use crate::peek::{step, Action, Input, Phase, POLL_MS};
    let spawned = std::thread::Builder::new().name("peek".into()).spawn(move || {
        let start = std::time::Instant::now();
        let mut phase = Phase::Away;
        loop {
            std::thread::sleep(Duration::from_millis(POLL_MS));
            let peeking = is_peeking();
            let strip = !peeking && !is_hidden() && STRIP_SHAPE.load(Ordering::SeqCst);
            if !strip && !peeking {
                phase = Phase::Resting;
                continue;
            }
            let extra = *PEEK_STRIP.lock().unwrap();
            let blocked = DRAGGING.load(Ordering::SeqCst) || sys::button_down() || FOLD.lock().unwrap().is_some();
            let input = Input {
                now: start.elapsed().as_millis() as u64,
                strip,
                peeking,
                over: sys::cursor_over(hwnd, extra.filter(|_| peeking)),
                blocked,
            };
            let (next, action) = step(phase, input);
            phase = next;
            match action {
                Action::Open => start_peek(&app),
                Action::Close => end_peek(&app),
                Action::None => {}
            }
        }
    });
    if let Err(e) = spawned {
        eprintln!("peepthis: peek thread failed: {e}");
    }
}

#[tauri::command]
pub fn get_mode() -> ModeView {
    mode_view()
}

/// The strip's click; `hidden` is the tray's alone.
#[tauri::command]
pub fn switch_mode(app: AppHandle, mode: Mode) -> Result<(), String> {
    if mode == Mode::Hidden {
        return Err("the widget hides from the tray".into());
    }
    set_mode(&app, mode);
    Ok(())
}

#[cfg(windows)]
pub fn hwnd<R: Runtime>(app: &AppHandle<R>) -> Option<isize> {
    app.get_webview_window(LABEL)?.hwnd().ok().map(|h| h.0 as isize)
}

#[cfg(not(windows))]
pub fn hwnd<R: Runtime>(_app: &AppHandle<R>) -> Option<isize> {
    None
}

/// Shows the panel and takes the keyboard (AV-24 inbox).
pub fn summon<R: Runtime>(app: &AppHandle<R>) {
    set_mode(app, Mode::Panel);
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
    // A saved `hidden` comes back as the strip: a launch always shows something.
    let strip = stored.as_ref().and_then(|s| s.mode) != Some(Mode::Panel);
    STRIP_SHAPE.store(strip, Ordering::SeqCst);
    let start = Layout::of(strip, anchor.unwrap_or(Anchor::TopRight)).initial_size(size.height);
    if let Some(((x, y), a)) = place_anchored(saved, anchor, &monitors, primary.as_ref(), start) {
        *ANCHOR.lock().unwrap() = a;
        w.set_position(PhysicalPosition::new(x, y))?;
    }
    #[cfg(windows)]
    {
        let hwnd = w.hwnd()?.0 as isize;
        sys::subclass(hwnd);
        spawn_peek(app.handle().clone(), hwnd);
    }
    spawn_saver(app.handle().clone());
    let moved = app.handle().clone();
    let _ = ANCHOR_MOVED.set(Box::new(move || emit_mode(&moved)));
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
            *TARGET.lock().unwrap() = None;
            let last = *LAST_CONTENT.lock().unwrap();
            if let Some((layout, len)) = last {
                let _ = apply_fit(w, layout, len);
            }
        }
        _ => {}
    }
}

/// Frontend reports the natural length of what it shows (CSS px): the panel's or column's height,
/// or the row's width. The window follows it up to the cap.
#[tauri::command]
pub fn fit_window(window: tauri::Window, height: f64, layout: Layout) -> Result<(), String> {
    if !height.is_finite() {
        return Err("height must be finite".into());
    }
    *LAST_CONTENT.lock().unwrap() = Some((layout, height));
    apply_fit(&window, layout, height).map_err(|e| e.to_string())
}

/// The frontend is about to animate from the shape on screen to `layout`, which needs `height`.
#[tauri::command]
pub fn begin_switch(window: tauri::Window, height: f64, layout: Layout) -> Result<Option<FoldView>, String> {
    if !height.is_finite() {
        return Err("height must be finite".into());
    }
    *LAST_CONTENT.lock().unwrap() = Some((layout, height));
    start_fold(&window, layout, height).map_err(|e| e.to_string())
}

/// Hit regions (CSS px in the union window) to play `gap_ms` apart beside the CSS animation.
#[tauri::command]
pub fn play_region(rects: Vec<[f64; 4]>, gap_ms: u32) {
    let Some((hwnd, scale)) = FOLD.lock().unwrap().as_ref().map(|f| (f.hwnd, f.scale)) else { return };
    let steps: Vec<Rect> = rects.into_iter().map(|r| cover(r, scale)).collect();
    #[cfg(windows)]
    crate::motion::play(steps, Duration::from_millis(gap_ms.clamp(1, 200) as u64), move |r| set_region(hwnd, &[*r]));
    #[cfg(not(windows))]
    let _ = (hwnd, steps, gap_ms);
}

/// True when the final resize is under a frozen picture: the frontend draws its beacon at the
/// window's bottom-right corner and waits for `uncovered`.
#[tauri::command]
pub fn end_switch(app: AppHandle) -> bool {
    finish_fold(None, Some(painted_signal(&app)))
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

    fn fit(content: f64, top: i32, m: &MonitorInfo) -> (Rect, bool) {
        target(Layout::Panel, content, (100, top), m, Anchor::Float)
    }

    #[test]
    fn strip_layout_follows_the_anchor() {
        use Anchor::*;
        for a in [Top, Bottom] {
            assert_eq!(Layout::of(true, a), Layout::Row);
        }
        for a in [Left, Right, TopLeft, TopRight, BottomLeft, BottomRight, Float] {
            assert_eq!(Layout::of(true, a), Layout::Column);
            assert_eq!(Layout::of(false, a), Layout::Panel);
        }
    }

    #[test]
    fn hotkey_toggles_panel_and_strip_and_recalls_hidden_as_panel() {
        assert_eq!(toggled(Mode::Panel, false), Mode::Strip);
        assert_eq!(toggled(Mode::Strip, false), Mode::Panel);
        assert_eq!(toggled(Mode::Hidden, false), Mode::Panel);
        // A peek reports `strip` but shows the panel: the key folds it.
        assert_eq!(toggled(Mode::Strip, true), Mode::Strip);
    }

    #[test]
    fn strip_and_panel_share_the_anchored_edge() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        let (margin, shadow) = (18, 9);
        let strip = target(Layout::Column, 300.0, (0, 0), &m, Anchor::BottomRight).0;
        let panel = target(Layout::Panel, 420.0, (strip.x, strip.y), &m, Anchor::BottomRight).0;
        assert_eq!(strip.w, to_phys(STRIP + SHADOW, 1.5));
        for r in [strip, panel] {
            assert_eq!((r.right() - shadow, r.bottom() - shadow), (1920 - margin, 1008 - margin));
        }
        let strip = target(Layout::Column, 300.0, (0, 200), &m, Anchor::Left).0;
        let panel = target(Layout::Panel, 420.0, (strip.x, strip.y), &m, Anchor::Left).0;
        assert_eq!((strip.x, strip.y), (margin, 200));
        assert_eq!((panel.x, panel.y), (margin, 200));
    }

    #[test]
    fn strip_row_sits_on_top_and_bottom_edges() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let top = target(Layout::Row, 420.0, (700, 500), &m, Anchor::Top).0;
        assert_eq!(top, Rect { x: 700, y: 12, w: 420, h: 68 });
        let bottom = target(Layout::Row, 420.0, (700, 0), &m, Anchor::Bottom).0;
        assert_eq!(bottom.bottom() - 6, 1040 - 12);
    }

    #[test]
    fn strip_floating_unfolds_from_its_corner_and_folds_back_home() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let home = (1800, 300);
        let strip = target(Layout::Column, 300.0, home, &m, Anchor::Float).0;
        assert_eq!((strip.x, strip.y), home);
        // The panel does not fit to the right of the strip, so it is pushed back inside.
        let panel = target(Layout::Panel, 420.0, home, &m, Anchor::Float).0;
        assert_eq!(panel.x, 1920 - 354);
        assert_eq!(target(Layout::Column, 300.0, home, &m, Anchor::Float).0, strip);
    }

    #[test]
    fn strip_length_is_floored_and_capped_by_the_work_area() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        assert_eq!(target(Layout::Column, 10.0, (0, 0), &m, Anchor::Float).0.h, to_phys(STRIP + SHADOW, 1.5));
        assert_eq!(target(Layout::Column, 5000.0, (0, 0), &m, Anchor::Float).0.h, 1008);
        assert!(!target(Layout::Column, 5000.0, (0, 0), &m, Anchor::Float).1);
    }

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
    fn fold_window_is_the_union_with_the_anchored_corner_shared() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        let strip = target(Layout::Column, 300.0, (0, 0), &m, Anchor::BottomRight).0;
        let panel = target(Layout::Panel, 420.0, (strip.x, strip.y), &m, Anchor::BottomRight).0;
        let (u, v) = fold_view(strip, panel, 1.5);
        assert_eq!(u, panel);
        assert_eq!(v.win, [360.0, 420.0]);
        assert_eq!(v.to, [0.0, 0.0, 360.0, 420.0]);
        // The strip sits in the union's bottom-right corner, where it already was on screen.
        assert_eq!(v.from, [360.0 - 68.0, 420.0 - 300.0, 68.0, 300.0]);
        assert_eq!(fold_view(panel, strip, 1.5).0, u);
    }

    #[test]
    fn fold_window_covers_a_panel_pushed_back_inside_the_work_area() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.0, 40);
        let strip = target(Layout::Column, 300.0, (1800, 300), &m, Anchor::Float).0;
        let panel = target(Layout::Panel, 420.0, (1800, 300), &m, Anchor::Float).0;
        let (u, v) = fold_view(strip, panel, 1.0);
        assert_eq!((u.x, u.right()), (panel.x, strip.right().max(panel.right())));
        assert_eq!(v.from[0], (strip.x - panel.x) as f64);
        assert_eq!(v.to[0], 0.0);
    }

    #[test]
    fn fold_region_rounds_outward_to_physical_pixels() {
        assert_eq!(cover([10.0, 20.0, 68.0, 100.0], 1.0), Rect { x: 10, y: 20, w: 68, h: 100 });
        let r = cover([10.5, 0.3, 67.7, 100.0], 1.25);
        assert_eq!((r.x, r.y), (13, 0));
        assert_eq!((r.right(), r.bottom()), (98, 126));
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
            let h = panel_height(5000.0, &m);
            assert_eq!(h, (0.7 * 1032.0 / scale).floor(), "scale {scale}");
            assert!(fit(5000.0, 0, &m).1);
            assert!(to_phys(h, scale) <= (0.7 * 1032.0) as i32 + 1);
        }
    }

    #[test]
    fn window_fit_follows_content_with_floor() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.25, 48);
        assert_eq!(panel_height(180.4, &m), 181.0);
        assert_eq!(panel_height(10.0, &m), MIN_HEIGHT);
        let (r, over) = fit(180.4, 12, &m);
        assert_eq!((r.y, r.h, r.w, over), (12, to_phys(181.0, 1.25), to_phys(WIDTH, 1.25), false));
    }

    #[test]
    fn window_fit_moves_up_to_stay_above_taskbar() {
        let m = mon("D1", 0, 0, 1920, 1080, 1.5, 72);
        let (r, _) = fit(400.0, 900, &m);
        assert_eq!(r.y, 1008 - to_phys(400.0 - SHADOW, 1.5));
        assert!(r.y >= 0);
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
        let card_bottom = |r: Rect| r.bottom() - 6;
        let short = target(Layout::Panel, 150.0, (1554, 500), &m, Anchor::BottomRight).0;
        let tall = target(Layout::Panel, 450.0, (short.x, short.y), &m, Anchor::BottomRight).0;
        assert_eq!(card_bottom(short), 1040 - 12);
        assert_eq!(card_bottom(tall), 1040 - 12);
        assert!(tall.y < short.y);
        // Top-anchored keeps its top.
        assert_eq!(target(Layout::Panel, 450.0, (1554, 12), &m, Anchor::TopRight).0.y, 12);
    }

    #[test]
    fn window_stored_file_reads_old_format_and_writes_anchor() {
        let old = r#"{"monitor":{"name":"D1","x":0,"y":0,"width":1920,"height":1080},"x":5,"y":6}"#;
        let s: Stored = serde_json::from_str(old).unwrap();
        assert_eq!((s.at.x, s.at.y, s.anchor), (5, 6, None));
        assert_eq!(s.mode, None);
        let new = Stored { anchor: Some(Anchor::Bottom), mode: Some(Mode::Strip), ..s };
        let text = serde_json::to_string(&new).unwrap();
        assert!(text.contains(r#""anchor":"bottom""#) && text.contains(r#""mode":"strip""#), "{text}");
        assert_eq!(serde_json::from_str::<Stored>(&text).unwrap(), new);
    }

    #[test]
    fn window_saved_state_round_trips_as_json() {
        let s = Saved { monitor: mon("\\\\.\\DISPLAY1", 0, 0, 1920, 1080, 1.0, 40).id, x: -3, y: 7 };
        let back: Saved = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back, s);
    }
}
