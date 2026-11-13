---
id: AV-19
title: Seven new peep accessories
status: in-progress
depends: [AV-04]
attended: true
---

# AV-19 — Seven new peep accessories

**What to build:** Drop in the owner's updated `peep.js`, which adds seven `accessory` values: `crybaby` `beanie` `crown` `shades` `flowers` `halo` `wizard`. Source and the owner's notes: `C:\Users\benbu\Downloads\peep-accessories-update\peep-accessories-update\` (`peep.js`, `ACCESSORIES-UPDATE.md`). The existing seven values and the rest of the API are unchanged.

The repo's `design/peep.js` has not been edited since the first handoff (it matches `src/vendor/peep.js` byte for byte, and the new file is a pure superset of it), so the owner's notes say to replace the file outright.

- [x] Replace `design/peep.js` and `src/vendor/peep.js` with the new file; the two stay byte-identical
- [x] `ACCESSORIES` in `src/identity.ts` grows to 14, in the order of the notes (existing seven first, then `crybaby beanie crown shades flowers halo wizard`)
- [x] Update the pinned outputs in `src/identity.test.ts`, plus the accessory list in spec §3.5 and in the `CLAUDE.md` Identity line
- [x] A check page (e.g. `?accessories` in dev) shows all 14 accessories at 96px on the widget background, for the owner to look at
- [ ] Stray pixels the owner spots go back to the owner as art notes. Per `CLAUDE.md`, we never redraw or tidy their art

**Done when:** `npm test` passes, the owner has looked at the check page, and either accepts the art or sends a corrected `peep.js`.

## Open decision (recommended answer)

- **Identity hash spread:** `(h >>> 16) % ACCESSORIES.length` now picks 1 of 14, which re-skins most existing projects once. The alternative is to keep the hash at 7 and use the new accessories only in `?demo`/manual use. **Recommended:** go to 14. Nothing has shipped yet, so no user has an attachment to their project's look.

## Notes

- The owner drew these without a visual check. Their notes say head space is roughly x 10–21, y 8–20 on the 32×32 grid, and the eyes sit at x 12–13 / 18–19, y 14–15. `shades` covers the eyes, so check it under `waiting` and `error` poses too.
- If the Downloads folder is gone when this runs, ask the owner for the file again. Do not reconstruct it from the diff in `ACCESSORIES-UPDATE.md`.

## Decisions taken without a ruling

- Identity hash goes to 14 (the recommendation above). The pinned outputs changed: `C:\Users\Dev\Projects\AgentView` is now `beanie`, `/home/dev/app` is `shades` and `""` is `halo`. Colours are unchanged, and this repo's own cwd keeps `none`.
- Check page: `npm run dev`, then `/?accessories`. Each accessory renders in the `idle`, `waiting` and `error` poses at 96px, cycling through the palette colours. It is dev-only and sits behind the query flag, like `?demo`.

## Art notes for the owner (from a headless screenshot, not fixed)

- **Antenna:** it pokes up through `beanie`, `crown`, `flowers` and `halo` (it sits inside the halo ring). For `wizard` it rides on the hat. This may be intended.
- **Flowers:** the crown sits low and partly overlaps the eyebrow/forehead line.
- **No visible stray pixels** at 96px. `shades` reads fine under `waiting` and `error`, though the error pose's X eyes are hidden by the lenses.

## Owed before done

The owner opens `npm run dev`, goes to `/?accessories`, and either accepts the art or sends a corrected `peep.js`. If they send a corrected file, replace both copies again.
