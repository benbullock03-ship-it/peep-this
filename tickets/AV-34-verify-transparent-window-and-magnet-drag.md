---
id: AV-34
title: Verify transparent window and live magnet drag
status: todo
depends: []
attended: true
---

# AV-34 — Verify transparent window and live magnet drag

**What to build:** Nothing shippable. A spike on a branch that proves the two platform assumptions the new surface rests on, and records the answers here for AV-35 and AV-38 (spec §12.1, §12.4).

- [ ] **Verify** a `transparent: true`, undecorated, always-on-top Tauri 2 window on Windows 11 draws a CSS-clipped card with nothing behind the cut corners, at 100%, 150% and 200% scaling. Record any WebView2 flags or config needed
- [ ] **Verify** whether clicks on the transparent margin reach the app underneath. If not, record the cheapest fix (window region, `set_ignore_cursor_events` by hit test, or a tighter window rect)
- [ ] **Verify** how to move the window while it is held so it can jump to a snap target and break free again: `WM_MOVING` subclass around `start_dragging`, or a manual drag driven by pointer events and `set_position`. Record which one works without flicker or lost mouse capture
- [ ] **Verify** position and size can change together (one `SetWindowPos`) so a window anchored on its right or bottom edge grows leftward or upward without a visible jump
- [ ] **Verify** resizing the window in 4 to 6 discrete steps over 160ms looks stepped, not torn
- [ ] Idle CPU with the transparent window stays inside the §8 budget

**Done when:** the owner dragged the spike window to all four edges of two monitors, felt it catch and release, and every Verify line has a recorded answer.

Decide: if no drag method gives a live magnet without flicker, fall back to snap on release (recommended fallback; AV-38 then drops the break-free rule).
