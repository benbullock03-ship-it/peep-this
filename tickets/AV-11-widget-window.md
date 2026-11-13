---
id: AV-11
title: Frameless docked always-on-top window
status: in-progress
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

## Decisions taken without a ruling

- Saved position is the outer top-left in **physical** pixels plus the monitor's name, physical position and size. Window size is logical (360 wide) and converted with the target monitor's scale at placement time.
- Saved spot is used only if a monitor with the same name, position and size exists and the saved rect overlaps its work area; it is then clamped fully inside. Otherwise: default dock on the primary (first monitor if no primary).
- Default dock is 12 logical px in from the work area's right and top edges, not flush, so the Win11 shadow and corner show.
- Height: the frontend reports header + list natural height via `fit_window`; Rust caps it at 70% of the current monitor's work area (min 48px). If growth would cross the work area's bottom, the window moves up; it does not move back down when it shrinks.
- Window starts hidden and shows on the first `fit_window` call (2s fallback show), so there is no size jump at launch.
- State file is `%APPDATA%\AgentView\window.json` (from `APPDATA`, not Tauri's identifier-based config dir), written via temp file + rename, debounced 500ms after the last `Moved`.
- `resizable: false`; window width is fixed.

## Owed before done

- Owner check in `npm run tauri dev`: frameless, on top, no taskbar entry, docked top-right inside the taskbar; drag by header; restart and it returns in place; list scrolls past ~70% height; repeat at 125%/150% scaling. `cargo test window` passes (15).
