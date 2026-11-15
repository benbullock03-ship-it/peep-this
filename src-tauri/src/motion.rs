//! Motion helpers for the window (spec §12.3). The strip/panel switch animates in CSS inside a
//! window held on the union of both shapes (AV-39); this side plays the hit region alongside it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static GENERATION: AtomicU64 = AtomicU64::new(0);

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

/// Stops a running `play` before its next step.
pub fn cancel() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// The step due `elapsed` after the start, holding on the last one.
pub fn step_at(elapsed: Duration, gap: Duration, len: usize) -> usize {
    ((elapsed.as_nanos() / gap.as_nanos().max(1)) as usize).min(len.saturating_sub(1))
}

/// Applies each step `gap` apart on a worker thread. Steps are picked by the clock, so a late
/// wake-up skips ahead instead of falling behind the CSS animation. A newer call cancels this one.
pub fn play<T: Send + 'static>(steps: Vec<T>, gap: Duration, apply: impl Fn(&T) + Send + 'static) {
    let me = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    if steps.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let start = Instant::now();
        let mut last = None;
        loop {
            if GENERATION.load(Ordering::SeqCst) != me {
                return;
            }
            let i = step_at(start.elapsed(), gap, steps.len());
            if last != Some(i) {
                apply(&steps[i]);
                last = Some(i);
            }
            if i + 1 == steps.len() {
                return;
            }
            std::thread::sleep((gap * (i as u32 + 1)).saturating_sub(start.elapsed()));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    const GAP: Duration = Duration::from_millis(20);

    #[test]
    fn motion_step_follows_the_clock_and_holds_on_the_last() {
        assert_eq!(step_at(Duration::ZERO, GAP, 12), 0);
        assert_eq!(step_at(Duration::from_millis(19), GAP, 12), 0);
        assert_eq!(step_at(Duration::from_millis(20), GAP, 12), 1);
        // A wake-up 50ms late lands on the step that is due, not the next in line.
        assert_eq!(step_at(Duration::from_millis(110), GAP, 12), 5);
        assert_eq!(step_at(Duration::from_secs(5), GAP, 12), 11);
        assert_eq!(step_at(Duration::from_secs(5), Duration::ZERO, 3), 2);
        assert_eq!(step_at(Duration::ZERO, GAP, 0), 0);
    }

    #[test]
    fn motion_play_ends_on_the_last_step_in_order() {
        let (tx, rx) = mpsc::channel();
        play(vec![1, 2, 3, 4], Duration::from_millis(2), move |s| tx.send(*s).unwrap());
        let seen: Vec<i32> = rx.iter().collect();
        assert_eq!(seen.last(), Some(&4));
        assert!(seen.windows(2).all(|w| w[0] < w[1]), "{seen:?}");
    }
}
