---
id: AV-35
title: Pixel-cut card chrome
status: in-progress
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
- [ ] Crisp at 100%, 150% and 200% scaling: no blurred border or shadow edges
- [x] Unit tests for the card-rect maths
- [x] Clicks on the shadow-free transparent area pass through: `SetWindowRgn` to card ∪ shadow, reapplied on every resize (AV-34 answer)
- [ ] Idle CPU on a release build inside §8 (AV-34 measured 1.61% on debug with two windows open, which doesn't count)

**Done when:** the owner saw the card over a light window and a dark one in `npm run tauri dev` and accepted the look.

## Owed before done 2026-11-14

Built: `tauri.conf.json` (`transparent`, `shadow: false`), `styles.css` (transparent `html`/`body`), `window.rs` (`SHADOW`, card-rect `place`/`fit`, `hit_region` + `SetWindowRgn` in `apply_fit`), spec §3.1. Tests: `window_docked_card_keeps_margin_and_shadow_overhangs_it`, `window_card_may_touch_work_edge_with_shadow_past_it`, `window_saved_position_from_before_av35_restores_same_card_spot`, `window_hit_region_is_card_plus_offset_shadow`, and the updated dock/fit tests.

Checked by Claude at 100%: a screen capture shows the editor through the window and a 6px shadow band darkening it (`#191A1B` to `#0E0E0F`). `WindowFromPoint` on the two transparent corners returns the window underneath; card and shadow return the widget.

Owed:
- Owner: card over a light and a dark window; crisp at 150% and 200%.
- **CPU:** release build, 3 live sessions, 2 runs of 30s each. Transparent 0.91% and 1.07%, opaque 1.15% and 1.22%. Transparency costs nothing measurable. The ~1% total comes from peeps animating for live sessions and was there before this ticket. It needs its own ticket if §8 "idle" means sessions busy rather than idle.

