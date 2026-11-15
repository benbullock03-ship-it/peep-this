---
id: AV-52
title: Light-theme chip text below AA contrast on the card background
status: todo
depends: [AV-51]
attended: false
---

# AV-52: Light-theme chip text below AA contrast on the card background

**What to fix:** In the light theme, coloured chip text on approval and question cards is under the WCAG AA minimum of 4.5:1 for small text. Found by the AV-51 `/shoot` proof run and ruled a real fail by the owner on 2026-11-15.

Measured: `.card-tier` is `--ps-accent` `#8a5a00`, 10px weight 500, on the card's `--ps-desk` `#e6ddcd` (`.card` background, `styles.css` ~1411): **4.40:1**. Dark theme is 13.52:1 and is not in scope.

The cause is the shared light token, so the fail is not only the `question` chip. By the same arithmetic (not yet measured in the page), light `--red` `#c22f2f` on `--ps-desk` is about 4.2:1, so the `destructive` chip fails too.

- [ ] Measure first with `/shoot` on `?demo`, light theme: every coloured text on a card or chip (`.card-tier` for `safe`, `shell`, `destructive`, `question`; `.card-risk`; `.card-agent`; trail marks; the title-bar count) against its effective background. List each ratio in this ticket
- [ ] Darken the light-theme tokens that fail, in `:root[data-theme="light"]` only. Recommended: `--ps-accent` `#8a5a00` → `#805300` (about 4.9:1 on `--ps-desk`, and higher on `--ps-panel`); `--red` to the nearest shade at or above 4.5:1 on `--ps-desk`. Keep the hue; only the lightness moves
- [ ] Check what else the changed tokens reach: `.card-allow` (white on accent; darker only raises it), focus outlines, `--amber` users if it changes, `--st-error` bubbles (`--red` is `var(--st-error)`)
- [ ] Rerun `/shoot` in light and dark at 360 and 1280: every measured ratio is at least 4.5:1, and dark-theme values are unchanged

**Done when:** `/shoot` reports no light-theme text under 4.5:1 on the cards and chips, dark theme is unchanged, and the colours still read as the same yellow/amber and red.

## Decisions taken without a ruling

- **Change the tokens, not the chip's background.** The card sits on `--ps-desk` by design (AV-44 shell). Darkening the light token fixes every user of it at once.
- **Light theme only.** The dark palette comes from the design reference and passes.
- **Not attended:** a colour-lightness fix with a measured target. The owner rules if the new shades read as a different colour.
