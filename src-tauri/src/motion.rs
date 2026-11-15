//! AV-37: stepped motion for the window (spec §12.3). Moves and resizes run in `FRAMES` discrete
//! steps over `DURATION`, matching the CSS `--motion-*` tokens; reduced motion jumps straight there.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::window::Rect;

/// Matches `--motion-frames` in styles.css.
pub const FRAMES: u32 = 5;
/// Matches `--motion-ms` in styles.css.
pub const DURATION: Duration = Duration::from_millis(160);

static GENERATION: AtomicU64 = AtomicU64::new(0);

/// The rects to show in order, ending exactly on `to`. One rect when `reduced`.
pub fn frames(from: Rect, to: Rect, n: u32, reduced: bool) -> Vec<Rect> {
    if reduced || n <= 1 || from == to {
        return vec![to];
    }
    let lerp = |a: i32, b: i32, i: u32| a + ((b - a) as f64 * i as f64 / n as f64).round() as i32;
    (1..=n)
        .map(|i| Rect { x: lerp(from.x, to.x, i), y: lerp(from.y, to.y, i), w: lerp(from.w, to.w, i), h: lerp(from.h, to.h, i) })
        .collect()
}

/// Windows "Show animations in Windows"; WebView2 maps the same switch to `prefers-reduced-motion`.
#[cfg(windows)]
pub fn reduced_motion() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION};
    let mut on: i32 = 1;
    let ok = unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, &mut on as *mut i32 as *mut _, 0) };
    ok != 0 && on == 0
}

#[cfg(not(windows))]
pub fn reduced_motion() -> bool {
    false
}

/// Stops a running `animate` before its next frame.
pub fn cancel() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// Steps the window from `from` to `to` on a worker thread, one `SetWindowPos` per frame (AV-34:
/// a single call per frame does not jump). `after` runs with each applied rect, e.g. to reapply the
/// hit region. A newer call cancels a running one.
#[cfg(windows)]
pub fn animate(hwnd: isize, from: Rect, to: Rect, after: impl Fn(Rect) + Send + 'static) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};
    let me = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let steps = frames(from, to, FRAMES, reduced_motion());
    let gap = DURATION / FRAMES;
    std::thread::spawn(move || {
        for (i, r) in steps.iter().enumerate() {
            if GENERATION.load(Ordering::SeqCst) != me {
                return;
            }
            if i > 0 {
                std::thread::sleep(gap);
            }
            unsafe { SetWindowPos(hwnd as _, std::ptr::null_mut(), r.x, r.y, r.w, r.h, SWP_NOZORDER | SWP_NOACTIVATE) };
            after(*r);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Rect = Rect { x: 100, y: 200, w: 360, h: 100 };
    const B: Rect = Rect { x: 100, y: 150, w: 360, h: 351 };

    #[test]
    fn motion_frames_step_evenly_and_end_exactly_on_target() {
        let f = frames(A, B, 5, false);
        assert_eq!(f.len(), 5);
        assert_eq!(*f.last().unwrap(), B);
        let hs: Vec<i32> = f.iter().map(|r| r.h).collect();
        assert_eq!(hs, vec![150, 200, 251, 301, 351]);
        assert_eq!(f[0].y, 190);
    }

    #[test]
    fn motion_reduced_or_unchanged_is_one_jump() {
        assert_eq!(frames(A, B, 5, true), vec![B]);
        assert_eq!(frames(A, A, 5, false), vec![A]);
        assert_eq!(frames(A, B, 1, false), vec![B]);
    }

    #[test]
    fn motion_frame_count_matches_spec_range() {
        assert!((4..=6).contains(&FRAMES));
        assert_eq!(DURATION.as_millis(), 160);
    }
}
