---
id: AV-16
title: Bundled fonts and visual polish pass
status: in-progress
depends: [AV-04, AV-14, AV-15]
attended: true
---

# AV-16 — Bundled fonts and visual polish pass

**What to build:** Fredoka, IBM Plex Sans and IBM Plex Mono bundled locally (no Google Fonts at runtime), and a pass making the widget match `design/peep-this.dc.html`'s monitor section at widget scale.

- [x] Fonts self-hosted under `src/assets/fonts`, OFL licences included
- [x] Spacing, borders and colours from the design tokens in `src/styles.css`
- [x] Row hover state; entrance animation for new rows; motion kept subtle
- [ ] Screenshots at 100% and 150% scaling reviewed with the owner

**Done when:** the owner signs off on the look.

## Decisions taken without a ruling

- Started before the owner's live check of AV-14 and AV-15, because the owner asked for this ticket by name.
- Fonts come from @fontsource 5.3.0: the weights the design page loads (Fredoka 500/600, Plex Sans 400/500/600, Plex Mono 400/500), latin and latin-ext subsets, woff2 only, `font-display: block` (local files, so no flash of fallback). Licences are `src/assets/fonts/OFL-*.txt`.
- The header title now matches the design page's header: Fredoka 600 in the accent yellow, -0.01em tracking.
- Entrance: new rows fade up 4px over 320ms. On first load the rows stagger 40ms apart, capped at 8. Off under `prefers-reduced-motion`.
- The expanded row's middle cut dropped from 38 to 36 chars, because Plex Mono is wider than the fallback it replaces.
- Row sizes stay at widget scale (13/12px text, 12px radius) rather than the mock's 14px/16px. The mock is a 1160px page.

## Owed before done

Built 2026-11-13: `src/fonts.css` + `src/assets/fonts/`, tokens (`--track`, `--hover`, `--amber`, `--radius`, `--display`, `--ease-out`) and entrance in `src/styles.css`, `enter()` in `src/rows.ts`. A headless check of `?demo` loaded all five faces with zero non-localhost requests. Owed: the owner reviews screenshots at 100% and 150% (and in `npm run tauri dev`) and signs off.
