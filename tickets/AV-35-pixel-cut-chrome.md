---
id: AV-35
title: Pixel-cut card chrome
status: todo
depends: [AV-34]
attended: true
---

# AV-35 — Pixel-cut card chrome

**What to build:** The panel stops being an opaque square window and becomes a pixel-cut card floating on a transparent window (spec §12.1). Rows, header and behaviour are unchanged.

- [ ] Window is transparent; `html`/`body` have no background; the card carries `#141218`
- [ ] Card corners are three 2px stairs (6px corner), with a 2px border in the existing border colour following the stairs
- [ ] Hard shadow: solid `rgba(0,0,0,0.5)`, offset 4px right and 4px down, same stepped outline, no blur
- [ ] No edge light: border and shadow never change with session state
- [ ] `window.rs` separates the card rect from the window rect; dock margin, `clamp_into`, `fit` and the saved position all use the card rect, so the shadow may overhang the margin but the card never does
- [ ] A saved position from before this change still restores to the same card position
- [ ] The inner `.list` panel keeps or loses its own radius so nothing smooth-rounded sits inside the stepped card (take the look that matches the reference mock closest and record it)
- [ ] Crisp at 100%, 150% and 200% scaling: no blurred stairs
- [ ] Unit tests for the card-rect maths

**Done when:** the owner saw the card over a light window and a dark one in `npm run tauri dev` and accepted the look.

Decide: build the stairs with `clip-path: polygon()` plus a second clipped element for the border, or with a 9-slice `border-image` (recommended: `clip-path`; no image asset, scales by a CSS variable).
