---
id: AV-52
title: Light-theme chip text below AA contrast on the card background
status: done
depends: [AV-51]
attended: false
---

# AV-52: Light-theme chip text below AA contrast on the card background

**What to fix:** In the light theme, coloured chip text on approval and question cards is under the WCAG AA minimum of 4.5:1 for small text. Found by the AV-51 `/shoot` proof run and ruled a real fail by the owner on 2026-11-15.

Measured: `.card-tier` is `--ps-accent` `#8a5a00`, 10px weight 500, on the card's `--ps-desk` `#e6ddcd` (`.card` background, `styles.css` ~1411): **4.40:1**. Dark theme is 13.52:1 and is not in scope.

The cause is the shared light token, so the fail is not only the `question` chip. By the same arithmetic (not yet measured in the page), light `--red` `#c22f2f` on `--ps-desk` is about 4.2:1, so the `destructive` chip fails too.

- [x] Measure first with `/shoot` on `?demo`, light theme: every coloured text on a card or chip (`.card-tier` for `safe`, `shell`, `destructive`, `question`; `.card-risk`; `.card-agent`; trail marks; the title-bar count) against its effective background. List each ratio in this ticket
- [x] Darken the light-theme tokens that fail, in `:root[data-theme="light"]` only. Recommended: `--ps-accent` `#8a5a00` → `#805300` (about 4.9:1 on `--ps-desk`, and higher on `--ps-panel`); `--red` to the nearest shade at or above 4.5:1 on `--ps-desk`. Keep the hue; only the lightness moves
- [x] Check what else the changed tokens reach: `.card-allow` (white on accent; darker only raises it), focus outlines, `--amber` users if it changes, `--st-error` bubbles (`--red` is `var(--st-error)`)
- [x] Rerun `/shoot` in light and dark at 360 and 1280: every measured ratio is at least 4.5:1, and dark-theme values are unchanged

**Done when:** `/shoot` reports no light-theme text under 4.5:1 on the cards and chips, dark theme is unchanged, and the colours still read as the same yellow/amber and red.

## Measured 2026-11-15 (light, `?demo`, before → after)

| Text | Token | Background | Before | After |
|---|---|---|---|---|
| `.card-tier` safe / shell / question | `--ps-accent` | `--ps-desk` | 4.40 | 4.94 |
| `.card-tier` destructive | `--red` | `--ps-desk` | 4.17 | 4.52 |
| `.card-risk` shell / safe | `--amber` | amber 18% on `--ps-panel` | 3.87 | 4.54 |
| `.card-risk` destructive | `--risk-text` | `--red` 22% on `--ps-panel` | 5.44 (arith.) | 5.40 |
| `.card-agent`, `.card-note`, `.card-cwd` | `--ps-dim` | `--ps-desk` | 3.56 | 4.54 |
| `.card-target` | `--ps-muted` | `--ps-desk` | 4.71 | 4.71 |
| `.ps-pending` (title-bar count) | white | `--st-waiting` | 5.05 | 5.05 |
| `.ps-title` | `--ps-accent` | `--ps-panel` | 5.79 | 6.50 |
| Trail marks (allowed / denied / other) | accent / red / dim | `--ps-panel-2` worst case | 4.92 / 4.66 / 3.98 (arith.) | 5.52 / 5.05 / 5.07 (arith.) |

The demo has no decision trail, so trail marks are computed, not measured in the page. Page has no horizontal overflow at 360 or 1280 in either theme.

## Decisions taken without a ruling

- **Change the tokens, not the chip's background.** The card sits on `--ps-desk` by design (AV-44 shell). Darkening the light token fixes every user of it at once.
- **Light theme only.** The dark palette comes from the design reference and passes.
- **Not attended:** a colour-lightness fix with a measured target. The owner rules if the new shades read as a different colour.
- **Also darkened `--amber` and `--ps-dim`.** The measure step found `.card-risk` on shell/safe cards (3.87) and the grey `.card-agent`/note/cwd text (3.56) failing too. Both moved to the nearest same-hue shade at 4.5:1. Cost: light `--ps-dim` `#665f70` is now almost the same as `--ps-muted` `#645c70`, so the dim/muted step barely shows in light. Overrule by darkening `--ps-muted` as well if the step matters.
- **Shades:** `--ps-accent` `#805300`, `--st-error` (so `--red`) `#b82d2d`, `--amber` `150, 81, 0`, `--ps-dim` `#665f70`. `--st-waiting` stays `#a85b00`: it is not text on a card.

## Done 2026-11-15

`src/styles.css` `:root[data-theme="light"]` carries the four new shades; the dark block is untouched. `/shoot` on `?demo` measured light minimum 4.52 at 360 and 1280; dark tokens read back as `#f7d64a` / `#ff7a7a` / `#7d7889` / `255, 178, 62`, with the tier chip at 13.52 as before. Screenshots of the destructive and shell cards in light at 360: red still reads red, accent still reads amber. `npm run build` and `npm test` (184) pass.
