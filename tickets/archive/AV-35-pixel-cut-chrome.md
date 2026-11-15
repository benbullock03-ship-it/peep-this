---
id: AV-35
title: Pixel-cut card chrome
status: done
depends: [AV-34]
attended: true
---

# AV-35 — Pixel-cut card chrome

**What to build:** The panel stops being an opaque square window and becomes the AV-44 pixel card floating on a transparent window (spec §12.1). Rows, header and behaviour are unchanged.

> Owner ruling 2026-11-14: the AV-44 "Pixel desktop" design replaces the three-stair corners. AV-44 draws the square card, 2px border and 6px hard shadow inside today's opaque window; this ticket makes the window transparent so the shadow falls on the desktop.

- [x] Window is transparent; `html`/`body` have no background; the card carries the panel colour
- [x] Card is square-cornered with a 2px border in the text colour (AV-44's `.ps--pixel`)
- [x] Hard shadow: solid `rgba(0,0,0,0.45)`, offset 6px right and 6px down, no blur
- [x] No edge light: border and shadow never change with session state
- [x] `window.rs` separates the card rect from the window rect; dock margin, `clamp_into`, `fit` and the saved position all use the card rect, so the shadow may overhang the margin but the card never does
- [x] A saved position from before this change still restores to the same card position
- [x] Crisp at 100%, 150% and 200% scaling: no blurred border or shadow edges
- [x] Unit tests for the card-rect maths
- [x] Clicks on the shadow-free transparent area pass through: `SetWindowRgn` to card ∪ shadow, reapplied on every resize (AV-34 answer)
- [x] Idle CPU on a release build inside §8 (AV-34 measured 1.61% on debug with two windows open, which doesn't count). Transparency adds nothing; the ~1% from peep animation moved to AV-46.

**Done when:** the owner saw the card over a light window and a dark one in `npm run tauri dev` and accepted the look.

## Build notes 2026-11-14

Built: `tauri.conf.json` (`transparent`, `shadow: false`), `styles.css` (transparent `html`/`body`), `window.rs` (`SHADOW`, card-rect `place`/`fit`, `hit_region` + `SetWindowRgn` in `apply_fit`), spec §3.1. Tests: `window_docked_card_keeps_margin_and_shadow_overhangs_it`, `window_card_may_touch_work_edge_with_shadow_past_it`, `window_saved_position_from_before_av35_restores_same_card_spot`, `window_hit_region_is_card_plus_offset_shadow`, and the updated dock/fit tests.

Checked by Claude at 100%: a screen capture shows the editor through the window and a 6px shadow band darkening it (`#191A1B` to `#0E0E0F`). `WindowFromPoint` on the two transparent corners returns the window underneath; card and shadow return the widget.

Live check 2026-11-14 (Claude, at the owner's request): card over a dark editor and over a white topmost window at 100%. Both are crisp, and the shadow falls cleanly on each with no fringe.

## Done 2026-11-15

Scaling check (Claude, with the owner's yes): I switched the display live via `DisplayConfigSetDeviceInfo` to 150%, then 200%, then back to 100%, capturing the dev widget at each.
- Window size: 360×268, then 540×399, then 720×536, so the region and the fit scale exactly.
- Border: 2px, then 3px, then 4px, of solid `#ECE9F1`, with no in-between shades on either side.
- Shadow at 150%: 9px of solid `#0E0E0F` over the dark editor.
- Shadow at 200%: the edge is hard. Past it the shadow darkens the screen-edge content beneath, which isn't flat, so its width can't be read from that sample.
- Text and peeps are sharp at both scales.

The display is back at 100%.

The CPU line is ticked for this ticket's question: transparency costs nothing.

- **CPU:** release build, 3 live sessions, 2 runs of 30s each. Transparent 0.91% and 1.07%, opaque 1.15% and 1.22%. Transparency costs nothing measurable. The ~1% total comes from peeps animating for live sessions and was there before this ticket. It needs its own ticket if §8 "idle" means sessions busy rather than idle.

