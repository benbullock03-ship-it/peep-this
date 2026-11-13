---
id: AV-19
title: Seven new peep accessories
status: todo
depends: [AV-04]
attended: true
---

# AV-19 — Seven new peep accessories

**What to build:** Drop in the owner's updated `peep.js`, which adds seven `accessory` values: `crybaby` `beanie` `crown` `shades` `flowers` `halo` `wizard`. Source and the owner's notes: `C:\Users\benbu\Downloads\peep-accessories-update\peep-accessories-update\` (`peep.js`, `ACCESSORIES-UPDATE.md`). The existing seven values and the rest of the API are unchanged.

The repo's `design/peep.js` has not been edited since the first handoff (it matches `src/vendor/peep.js` byte for byte, and the new file is a pure superset of it), so the owner's notes say to replace the file outright.

- [ ] Replace `design/peep.js` and `src/vendor/peep.js` with the new file; the two stay byte-identical
- [ ] `ACCESSORIES` in `src/identity.ts` grows to 14, in the order of the notes (existing seven first, then `crybaby beanie crown shades flowers halo wizard`)
- [ ] Update the pinned outputs in `src/identity.test.ts`, plus the accessory list in spec §3.5 and in the `CLAUDE.md` Identity line
- [ ] A check page (e.g. `?accessories` in dev) shows all 14 accessories at 96px on the widget background, for the owner to look at
- [ ] Stray pixels the owner spots go back to the owner as art notes. Per `CLAUDE.md`, we never redraw or tidy their art

**Done when:** `npm test` passes, the owner has looked at the check page, and either accepts the art or sends a corrected `peep.js`.

## Open decision (recommended answer)

- **Identity hash spread:** `(h >>> 16) % ACCESSORIES.length` now picks 1 of 14, which re-skins most existing projects once. The alternative is to keep the hash at 7 and use the new accessories only in `?demo`/manual use. **Recommended:** go to 14. Nothing has shipped yet, so no user has an attachment to their project's look.

## Notes

- The owner drew these without a visual check. Their notes say head space is roughly x 10–21, y 8–20 on the 32×32 grid, and the eyes sit at x 12–13 / 18–19, y 14–15. `shades` covers the eyes, so check it under `waiting` and `error` poses too.
- If the Downloads folder is gone when this runs, ask the owner for the file again. Do not reconstruct it from the diff in `ACCESSORIES-UPDATE.md`.
