---
id: AV-35
title: Pixel-cut card chrome
status: todo
depends: [AV-34]
attended: true
---

# AV-35 — Pixel-cut card chrome

**What to build:** The panel stops being an opaque square window and becomes the AV-44 pixel card floating on a transparent window (spec §12.1). Rows, header and behaviour are unchanged.

> Owner ruling 2026-11-14: the AV-44 "Pixel desktop" design replaces the three-stair corners. AV-44 draws the square card, 2px border and 6px hard shadow inside today's opaque window; this ticket makes the window transparent so the shadow falls on the desktop.

- [ ] Window is transparent; `html`/`body` have no background; the card carries the panel colour
- [ ] Card is square-cornered with a 2px border in the text colour (AV-44's `.ps--pixel`)
- [ ] Hard shadow: solid `rgba(0,0,0,0.45)`, offset 6px right and 6px down, no blur
- [ ] No edge light: border and shadow never change with session state
- [ ] `window.rs` separates the card rect from the window rect; dock margin, `clamp_into`, `fit` and the saved position all use the card rect, so the shadow may overhang the margin but the card never does
- [ ] A saved position from before this change still restores to the same card position
- [ ] Crisp at 100%, 150% and 200% scaling: no blurred border or shadow edges
- [ ] Unit tests for the card-rect maths

**Done when:** the owner saw the card over a light window and a dark one in `npm run tauri dev` and accepted the look.
