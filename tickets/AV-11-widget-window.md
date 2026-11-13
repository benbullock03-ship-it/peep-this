---
id: AV-11
title: Frameless docked always-on-top window
status: todo
depends: [AV-01]
attended: false
---

# AV-11 — Frameless docked always-on-top window

**What to build:** Window behaviour from spec §3.1: frameless, always on top, skip taskbar, ~360px wide on the right edge of the primary monitor, height fits rows to a cap, draggable header, position persisted per monitor.

- [ ] Height follows content up to 70% of the work area, then the list scrolls
- [ ] Saved position restored; falls back to the default dock if its monitor is gone
- [ ] Respects the taskbar (work area, not full screen)
- [ ] Correct at 100%, 125% and 150% display scaling

**Done when:** the owner has dragged it, restarted, and seen it come back in place.
