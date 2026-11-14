---
id: AV-38
title: Magnetic snap to edges and corners
status: todo
depends: [AV-34, AV-35]
attended: false
---

# AV-38 — Magnetic snap to edges and corners

**What to build:** Dragging the widget near a work-area edge or corner makes it catch there while still held (spec §12.4). Built on the drag method AV-34 verified.

- [ ] Pure function `(card rect, monitors) -> snapped rect + anchor`, unit-tested: within 16 logical px of a work-area edge the card snaps to it with the 12px margin; two edges at once is a corner
- [ ] Live magnet: the window jumps to the snapped spot during the drag; moving more than 16px past it breaks free
- [ ] Works on every monitor, using that monitor's work area and scale; the taskbar is never covered
- [ ] Dropped away from any edge it floats where it was left
- [ ] The anchor (`left`, `right`, `top`, `bottom`, a corner, or `float`) is saved with the position and restored; a saved file without an anchor is read as `float` unless it sits on the default dock
- [ ] Height changes keep the anchored edge fixed: a bottom-anchored card grows upward
- [ ] First run still docks top-right, now recorded as the top-right corner anchor

**Done when:** the owner snapped the widget to all four edges and a corner on each monitor, pulled it free, restarted, and found it where it was left.
