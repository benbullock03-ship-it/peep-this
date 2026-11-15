---
id: AV-39
title: Peep strip collapsed state
status: in-progress
depends: [AV-35, AV-37, AV-38]
attended: false
---

# AV-39 — Peep strip collapsed state

**What to build:** A second visible form of the widget: a slim plate holding one 40px peep per session (spec §12.5).

> Owner ruling 2026-11-14: build the strip from the AV-44 design (`design/shell/`: `stripHTML`, `.ps-strip`, `.ps-strip-btn`): 62px plate, 40px peeps on mini shelves, attention cells tinted and ringed, a state dot per cell, the `‹` expand button at the top. Where the list below says 32px or stairs, the design wins. This ticket builds the strip and the switch between strip and panel; the hotkey (AV-40), hover peek (AV-41) and waiting flyout (AV-42) come after.

- [x] Backend holds the mode: `panel`, `strip` or `hidden`; persisted; the app launches in `strip`
- [x] Strip plate has the same border and hard shadow as the card, no border on the docked edge
- [x] One 40px peep per top-level session in the panel's sort order, with its real state, colour and accessory. No names, bars, LEDs, subagents or shells
- [x] More than 8 sessions: 8 peeps, then a `+N` tile in pixel type
- [x] A failed shell's red dot shows on its parent's peep once AV-30 exists (leave the hook, do not build shells here)
- [x] Orientation follows the anchor from AV-38: a column on the left and right edges, a row on the top and bottom edges, a column at corners and when floating
- [x] Strip and panel share the anchor: the panel unfolds from the strip away from the snapped edge, and the anchored edge never moves (timing: see the 2026-11-15 decision below)
- [ ] The whole plate drags the widget; a press that does not move is a click
- [x] Clicking the strip switches to the panel; a tray menu item switches back until AV-40 lands
- [ ] Peeps keep animating in the strip and stop when the mode is `hidden`
- [x] Vitest for the strip's session selection and `+N` count

**Done when:** the owner switched between strip and panel on a right edge, a top edge and floating, with nothing jumping, and restarted into the strip.

## Decisions taken without a ruling

- "Persisted" and "launches in `strip`" read together: the saved mode is restored, a first run or a file without a mode is `strip`, and a saved `hidden` comes back as `strip` so a launch always shows something. If the owner meant every launch is `strip`, it is one line in `window::setup`.
- The strip keeps the 12px margin on its snapped edge, the same line the panel sits on, because the anchored edge must not move. The plate still drops its border on that edge as the ticket says, so the open side faces a 12px gap, not the screen edge. The design drew it flush.
- Floating, the panel unfolds right and down from the strip's top-left and is pushed back inside the work area if it does not fit. Folding returns the strip to where it was before the unfold, unless the widget was dragged in between.
- Anything that needs rows switches to the panel: the inbox hotkey, the tray's settings item, a failed integration install. Nothing switches back on its own; AV-40 and AV-42 own that.
- Held approvals are still held in the strip, where no card shows until AV-42. The toast, the tinted cell and the inbox hotkey are the ways in; a focused terminal still gets its prompt at once. Alerts treat the strip as "row not in view", so a focused strip does not suppress a toast.
- Every cell and the `‹` button open the panel. Opening on that session's row (the design's `peepshell:select`) is not built.
- `+N` is set in Plex Mono 500, the face the shell already uses for its pixel-desktop labels; no pixel font is bundled.
- A row strip longer than the work area is cut at the work area. A column of 8 peeps and `+N` is about 580px.
- The tray item is "Collapse to strip", under Show/Hide.
- Owner asked 2026-11-15 for a button to get from the panel to the strip: the header has the design's `›` button (`#collapse-btn`), right of settings. This is AV-40's "collapse control in the header", built early; AV-40 still owes the hotkey it mirrors.
- Owner 2026-11-15 on the switch: "choppy; it needs to feel fluid yet nostalgic as you close and open the tray". The 5 `SetWindowPos` steps over 160ms are gone. A switch now sizes the window once to the union of the strip and panel rects, animates the card inside it (a frame that travels from one rect to the other with the anchored side still, old content fading out and new content fading in, both clipped to the frame), then drops the window to the final rect in one `SetWindowPos`. The hit region follows the card throughout. Reduced motion is still instant.
- **Rule exception:** this one transition does not follow the 4 to 6 frames / 160ms rule (spec §12.3). Row animations and everything else still do.
- **Owner's choice 2026-11-15: 12 steps over 240ms** ("go with a"), after seeing it beside a 220ms ease-out glide with a 2-frame snap and a zoom-rect outline. The other two and the dev picker are removed; `src/fold.ts` holds `FOLD_MS` and `FOLD_STEPS`.
- Each step shows the end of its interval, so the card moves on the click with no dead first frame.
- Owner saw a blink on open and close "sometimes". Cause, measured: WebView2 shows one blank frame when its window resizes, which a switch does twice (growing to the union before an unfold, shrinking after a fold); worse with a tall panel. Fix: `src-tauri/src/cover.rs` freezes a picture of the strip's screen rect in a click-through topmost window over each resize. First try dropped it two painted frames later; the owner still saw the blink on regular and tall panels, because two frames is not always enough. Now the page draws a 2px yellow beacon in the shadow's bottom-right corner once laid out at the new size, the backend watches that screen pixel, and only when it shows does the picture come off and (on an unfold) the card start moving. Gives up after 450ms. The match is loose (±40 per channel): this display reads the beacon back as `f3d75f`, not `f9d549`.
- `motion.rs` lost `animate`, `frames`, `FRAMES` and `DURATION`: nothing steps the window any more. It keeps `reduced_motion` and gained `play`, which runs the hit region by the clock.
- A switch that arrives mid-animation lands the one in flight at once, then animates from there. If the frontend never ends a switch, the backend ends it after 1.5s.
- The `design-tropes` skill the ticket workflow names is not installed in this environment; the look was checked against `design/shell/` and by screenshot.

## Owed before done

Built 2026-11-15. Code: `src-tauri/src/window.rs` (`Mode`, `Layout`, `target`, `set_mode`, `switch_mode`, `get_mode`, the `mode` event, mode in `window.json`; 5 new `strip_*` tests, fit tests moved onto `target`), `src-tauri/src/motion.rs` (`cancel`), `src-tauri/src/tray.rs`, `src/strip.ts` + `src/strip.test.ts` (7 tests), `src/stripPlate.ts`, `src/main.ts` (`startSurface`), `src/windowFit.ts`, `.ps-strip` in `src/styles.css`.

Seen by Claude in `npm run tauri dev` (one 2560×1440 monitor at 100%, bottom-right anchor): launched as a 68×170 strip with its card's right and bottom on the margin; one click unfolded it to the 360-wide panel with the same right and bottom; `window.json` recorded `strip`, then `panel`. Column and row plates were looked at in a browser (`?demo&strip`, `?demo&strip=row&anchor=top`).

Switch animation, seen by Claude 2026-11-15 in `npm run tauri dev` (2560×1440 at 100%), by clicking `›` and the strip and grabbing screen frames about 14ms apart: fold and unfold on bottom-right, right, top (row strip) and floating. Every frame had the window on the union rect with the anchored edges where they started, the hit region on the card, and the final rect equal to the old one. With the beacon in, 6 switches on bottom-right (236px panel) were sampled every 4 to 8ms at the strip's spot with no blank frame; the card starts moving about 40ms after the click. Before any cover, 3 of 3 unfolds of a 974px panel had a blank frame. A tall panel was not re-measured.

The owner then named the last cause: the strip showing at the panel's corner for an instant. On an unfold the window was resized before its region was set, so the old strip-shaped region sat at the union's top-left for a moment. The region is now set first. **Owner 2026-11-15, in `npm run tauri dev` after that fix: "looks good".** Which edges the owner tried was not said, so "Done when" (right, top, floating, and a restart into the strip) is still open.

Not seen live, so not claimed:
- Folding back from the tray item (the item is in the menu; driving the tray by script was unreliable).
- Dragging the plate by hand. A scripted header drag did turn the column into a row on the top edge. The drag box stays unticked.
- A switch interrupted mid-animation, a floating panel pushed back inside the work area (unit-tested only), 150% and 200% scale, and reduced motion.
- Peeps animating in the strip and stopping when hidden (unticked).
- The `+N` tile with more than 8 real sessions; the count is unit-tested.
- A restart into a saved `panel`, and light theme on the strip.

The owner's check in "Done when" closes the ticket.
