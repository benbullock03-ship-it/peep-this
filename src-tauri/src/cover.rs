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

/// The colour of the 2px beacon the frontend draws once it has laid out at the new size
/// (`.fold-beacon` in styles.css). Seeing it on screen means the blank frames are over.
pub const BEACON: (u8, u8, u8) = (249, 213, 73);
const BEACON_WAIT: Duration = Duration::from_millis(450);

/// The cover leaves this pixel of `r` bare so the beacon under it can be seen.
pub fn beacon_pixel(r: Rect) -> (i32, i32) {
    (r.x + r.w - 1, r.y + r.h - 1)
}

pub fn is_beacon(rgb: (u8, u8, u8)) -> bool {
    // Loose: the display's colour profile shifts what the screen reads back (f3d75f was seen).
    let near = |a: u8, b: u8| a.abs_diff(b) <= 40;
    near(rgb.0, BEACON.0) && near(rgb.1, BEACON.1) && near(rgb.2, BEACON.2)
}

/// Freezes `r` on screen, then takes the picture down once the beacon shows at `r`'s last pixel
/// (or after `BEACON_WAIT`) and calls `then`.
#[cfg(windows)]
pub fn show_until_painted(r: Rect, then: impl FnOnce() + Send + 'static) {
    show(Rect { h: r.h - 1, ..r });
    let (x, y) = beacon_pixel(r);
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        while start.elapsed() < BEACON_WAIT && !is_beacon(sys::pixel(x, y)) {
            std::thread::sleep(Duration::from_millis(2));
        }
        hide();
        then();
    });
}

#[cfg(not(windows))]
pub fn show_until_painted(_r: Rect, then: impl FnOnce() + Send + 'static) {
    then();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_beacon_is_the_last_pixel_and_matches_loosely() {
        assert_eq!(beacon_pixel(Rect { x: 100, y: 200, w: 68, h: 170 }), (167, 369));
        assert!(is_beacon(BEACON));
        assert!(is_beacon((243, 215, 95)));
        assert!(!is_beacon((27, 25, 33)));
        assert!(!is_beacon((255, 255, 255)));
    }
}

#[cfg(windows)]
mod sys {
    use super::Rect;
    use windows_sys::Win32::Foundation::{HWND, POINT, SIZE};
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, GetPixel, ReleaseDC, SelectObject,
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

    pub fn pixel(x: i32, y: i32) -> (u8, u8, u8) {
        unsafe {
            let screen = GetDC(std::ptr::null_mut());
            let c = GetPixel(screen, x, y);
            ReleaseDC(std::ptr::null_mut(), screen);
            // COLORREF is 0x00BBGGRR.
            (c as u8, (c >> 8) as u8, (c >> 16) as u8)
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
