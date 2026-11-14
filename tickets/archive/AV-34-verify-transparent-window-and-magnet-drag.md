---
id: AV-34
title: Verify transparent window and live magnet drag
status: done
depends: []
attended: true
---

# AV-34 — Verify transparent window and live magnet drag

**What to build:** Nothing shippable. A spike on a branch that proves the two platform assumptions the new surface rests on, and records the answers here for AV-35 and AV-38 (spec §12.1, §12.4).

- [x] **Verify** a `transparent: true`, undecorated, always-on-top Tauri 2 window on Windows 11 draws a CSS-clipped card with nothing behind the cut corners, at 100%, 150% and 200% scaling. Record any WebView2 flags or config needed
- [x] **Verify** whether clicks on the transparent margin reach the app underneath. If not, record the cheapest fix (window region, `set_ignore_cursor_events` by hit test, or a tighter window rect)
- [x] **Verify** how to move the window while it is held so it can jump to a snap target and break free again: `WM_MOVING` subclass around `start_dragging`, or a manual drag driven by pointer events and `set_position`. Record which one works without flicker or lost mouse capture
- [x] **Verify** position and size can change together (one `SetWindowPos`) so a window anchored on its right or bottom edge grows leftward or upward without a visible jump
- [x] **Verify** resizing the window in 4 to 6 discrete steps over 160ms looks stepped, not torn
- [ ] Idle CPU with the transparent window stays inside the §8 budget

**Done when:** the owner dragged the spike window to all four edges of two monitors, felt it catch and release, and every Verify line has a recorded answer.

Decide: if no drag method gives a live magnet without flicker, fall back to snap on release (recommended fallback; AV-38 then drops the break-free rule).

## Done 2026-11-14

Spike: branch `spike/AV-34` (d1ebcc2), `src-tauri/src/spike.rs` + `spike.html`, run with `PEEP_SPIKE=1 npm run tauri dev`. Owner tested on two monitors at 100/150/200%.

- **Transparency:** `WebviewWindowBuilder` with `.transparent(true).decorations(false).shadow(false)`, `html, body { background: transparent }`. Nothing drawn behind margin or the 6px hard shadow at any scale. No WebView2 flags needed.
- **Click-through:** the plain window rect swallows clicks on the transparent margin. Fix: `SetWindowRgn` to card ∪ shadow rect (window-relative physical px, scale from `GetDpiForWindow`). Reapply after every resize. Hit-test polling also works but costs a 30Hz thread; not taken.
- **Drag:** native drag (`data-tauri-drag-region` → `start_dragging`) plus `SetWindowSubclass` handling `WM_MOVING`: rewrite the proposed `RECT` so the card sits 12px from any work-area edge within 16px (`MonitorFromRect` nearest → `rcWork`). The proposed rect follows the cursor offset, so dragging past 16px breaks free with no extra state. No flicker, capture kept. Manual pointer drag also worked, but it isn't needed. Snap math and tests: `spike::snap`.
- **Anchored resize:** one `SetWindowPos(x, y, w, h, SWP_NOZORDER|SWP_NOACTIVATE)` grows upward from a bottom anchor cleanly. Tauri `set_size` then `set_position` shows a visible jump. AV-37/AV-38 must use the single call.
- **Stepped resize:** 5 one-call steps at 0/40/80/120/160ms looks stepped, not torn.
- **CPU (unticked):** 1.61% of a 12-thread machine over 20s, 505MB across 7 processes. That was a debug build with the main widget and spike open and 3 live sessions animating, so it does not test the budget. Re-measure on a release build in AV-35, which makes the main window transparent.
