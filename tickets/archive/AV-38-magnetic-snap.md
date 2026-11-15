---
id: AV-38
title: Magnetic snap to edges and corners
status: done
depends: [AV-34, AV-35]
attended: false
---

# AV-38 — Magnetic snap to edges and corners

**What to build:** Dragging the widget near a work-area edge or corner makes it catch there while still held (spec §12.4). Built on the drag method AV-34 verified.

- [x] Pure function `(card rect, monitors) -> snapped rect + anchor`, unit-tested: within 16 logical px of a work-area edge the card snaps to it with the 12px margin; two edges at once is a corner
- [x] Live magnet: the window jumps to the snapped spot during the drag; moving more than 16px past it breaks free
- [x] Works on every monitor, using that monitor's work area and scale; the taskbar is never covered
- [x] Dropped away from any edge it floats where it was left
- [x] The anchor (`left`, `right`, `top`, `bottom`, a corner, or `float`) is saved with the position and restored; a saved file without an anchor is read as `float` unless it sits on the default dock
- [x] Height changes keep the anchored edge fixed: a bottom-anchored card grows upward
- [x] First run still docks top-right, now recorded as the top-right corner anchor

**Done when:** the owner snapped the widget to all four edges and a corner on each monitor, pulled it free, restarted, and found it where it was left.

## Decisions taken without a ruling

- The magnet is measured from the snapped spot, as in the AV-34 spike the owner tried: the card catches while its edge is within 16px of the 12px-margin line.
- Windows builds each `WM_MOVING` proposal from the previous rect, so a snapped rect would stick and never break free (seen live). The handler rebuilds the raw rect from the cursor and the grab offset recorded at `WM_ENTERSIZEMOVE`.
- Dropping a card part-way off the work area (over the taskbar or a screen edge) pulls it back inside on release. While held it may hang over, so it can still be dragged to another monitor.
- On restore, anchored axes are recomputed from the anchor, not taken from the saved x/y. A bottom-anchored card launches on the bottom margin whatever height it starts at.
- `window.json` gains `"anchor"` through a flattened `Stored` wrapper, so older files still parse.
- `apply_fit` now sets size and position in one `SetWindowPos` (AV-34). Upward growth would otherwise flash a frame at the old top.

## Done 2026-11-15

Code: `src-tauri/src/snap.rs` (`snap`, `anchored_origin`, `Anchor`, 6 tests). `src-tauri/src/window.rs`: `place_anchored`, `fit_anchored`, the `Stored` file format, the `ANCHOR` state, the `sys` subclass (`WM_ENTERSIZEMOVE`, `WM_MOVING`, `WM_EXITSIZEMOVE`) and the one-call fit; 5 new tests cover the first-run anchor, the no-anchor read, bottom restore, upward growth and the old format. Cargo features `Win32_UI_HiDpi` and `Win32_UI_Shell`.

Live check (Claude, with real mouse drags on the title bar, one 2560×1440 monitor at 100%):
- Catching and letting go: dragged to 10px past the right snap spot, it held at x=2194 while still pressed. Pulled 10px it stayed caught; pulled 30px it came free.
- Every edge: top-left, top, left, bottom-right and bottom each saved the right anchor. A drop in the middle floated. A left drop 22px past the spot floated and was clamped inside.
- Upward growth: bottom-anchored, expanding a row grew the window from 316 to 537 tall upward, with the card's bottom fixed at 1380.
- Restart: an installed release build relaunched bottom-right at a different height, with the card's bottom still on the margin.

Not covered: a second monitor (only one is attached) and other scales live. `snap_uses_the_given_work_area_on_any_monitor` covers the maths, and the handler reads the per-window DPI and the nearest monitor's work area.
