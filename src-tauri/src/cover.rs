//! AV-39: WebView2 shows a blank frame when its window is resized. A cover is a frozen picture of
//! one screen rect, held on top (click-through) while the widget resizes underneath, then dropped.

use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::time::Duration;

use crate::window::Rect;

/// A cover nobody takes down comes down on its own.
const MAX_LIFE: Duration = Duration::from_millis(600);

static COVER: Mutex<Option<Sender<()>>> = Mutex::new(None);

/// Freezes what the screen shows in `r` (physical px) until `hide`. Returns once it is up.
#[cfg(windows)]
pub fn show(r: Rect) {
    hide();
    if r.w <= 0 || r.h <= 0 {
        return;
    }
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (ready_tx, ready_rx) = mpsc::channel::<()>();
    // The window lives and dies on this thread: a window's messages go to the thread that made it.
    std::thread::spawn(move || {
        let Some(c) = sys::create(r) else { return };
        let _ = ready_tx.send(());
        let born = std::time::Instant::now();
        while born.elapsed() < MAX_LIFE {
            sys::pump();
            match stop_rx.recv_timeout(Duration::from_millis(4)) {
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                _ => break,
            }
        }
        sys::destroy(c);
    });
    if ready_rx.recv_timeout(Duration::from_millis(200)).is_ok() {
        *COVER.lock().unwrap() = Some(stop_tx);
    }
}

#[cfg(not(windows))]
pub fn show(_r: Rect) {}

pub fn hide() {
    if let Some(tx) = COVER.lock().unwrap().take() {
        let _ = tx.send(());
    }
}

#[cfg(windows)]
mod sys {
    use super::Rect;
    use windows_sys::Win32::Foundation::{HWND, POINT, SIZE};
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
        AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC,
        SRCCOPY,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, PeekMessageW, ShowWindow, UpdateLayeredWindow, MSG,
        PM_REMOVE, SW_SHOWNOACTIVATE, ULW_ALPHA, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
        WS_EX_TRANSPARENT, WS_POPUP,
    };

    pub struct Cover {
        hwnd: HWND,
        mem: HDC,
        dib: HBITMAP,
    }

    pub fn create(r: Rect) -> Option<Cover> {
        unsafe {
            let screen = GetDC(std::ptr::null_mut());
            let mem = CreateCompatibleDC(screen);
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: r.w,
                // Negative: top-down rows.
                biHeight: -r.h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..std::mem::zeroed()
            };
            let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
            let dib = CreateDIBSection(mem, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
            if dib.is_null() || bits.is_null() {
                DeleteDC(mem);
                ReleaseDC(std::ptr::null_mut(), screen);
                return None;
            }
            SelectObject(mem, dib);
            BitBlt(mem, 0, 0, r.w, r.h, screen, r.x, r.y, SRCCOPY);
            // A screen copy has no alpha; a layered window needs it opaque.
            let px = std::slice::from_raw_parts_mut(bits as *mut u32, (r.w * r.h) as usize);
            px.iter_mut().for_each(|p| *p |= 0xFF00_0000);
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                windows_sys::w!("STATIC"),
                std::ptr::null(),
                WS_POPUP,
                r.x,
                r.y,
                r.w,
                r.h,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            let cover = Cover { hwnd, mem, dib };
            if hwnd.is_null() {
                ReleaseDC(std::ptr::null_mut(), screen);
                destroy(cover);
                return None;
            }
            let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
            UpdateLayeredWindow(hwnd, screen, &POINT { x: r.x, y: r.y }, &SIZE { cx: r.w, cy: r.h }, mem, &POINT { x: 0, y: 0 }, 0, &blend, ULW_ALPHA);
            ReleaseDC(std::ptr::null_mut(), screen);
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            Some(cover)
        }
    }

    pub fn pump() {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                DispatchMessageW(&msg);
            }
        }
    }

    pub fn destroy(c: Cover) {
        unsafe {
            if !c.hwnd.is_null() {
                DestroyWindow(c.hwnd);
            }
            DeleteObject(c.dib);
            DeleteDC(c.mem);
        }
    }
}
