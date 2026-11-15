---
id: AV-66
title: Inserted floppy on the plank and ticket-aware bubble
status: closed
depends: [AV-63, AV-64]
attended: true
---

# AV-66 — Inserted floppy and bubble

**What to build:** A row shows the running skill and the ticket, per the AV-63 design.

- [ ] Inserted floppy on the plank while a skill runs; eject frames on Stop (stepped motion, instant under reduced motion)
- [ ] Bubble shows ticket and skill ("AV-34: chip contrast · shooting") per the design's truncation rules
- [ ] Attention states (`waiting`, `error`, `stuck`) take over the bubble as today
- [ ] Vitest for the bubble text builder (with and without ticket, truncation, attention takeover)
- [ ] Shoot both themes and both widths

**Done when:** the owner has seen a live `/peep:ticket` run insert and eject the floppy in `npm run tauri dev`.

## Notes

- Built against the AV-63 drop while AV-63 is still `open` (not marked final by the owner).
- `src/bubble.ts` builds the bubble (`bubbleFor`, `bubbleLine`, `fillBubble`); `rows.ts` seats the floppy in `.ps-plank` from the sprite sheets copied to `src/assets/floppy/`. Tests: `src/bubble.test.ts`.
- Shoot (dark/light, 360/1280, reduced motion): DOM facts held after two fixes (chip/floppy overlap, stage bubble overflow). Eject class to removal 401ms; under reduced motion animation `none` and removal in 5ms. Container-query truncation checked by forcing body widths 250/150/80.
- Owed: owner sees a live `/peep:ticket` run insert and eject the floppy in `npm run tauri dev`.

## Decisions taken without a ruling

- Verbs: ducking, questioning (qt), ticketing, shooting, packing up, rotting (coderot); an unknown skill shows the blank disk and its own name as verb.
- Without a ticket, a running skill appends ` · verb` to the normal bubble line; a ticket without a title puts the normal line in the title slot.
- The ticket grammar holds while the ticket stays linked after Stop (verb gone), so a finished session reads `AV-34: title`, not "All done!".
- Truncation is CSS container queries on `.ps-body` (verb gone at ≤180px, id only at ≤90px); Vitest covers the parts, not the widths.
- The app plank is full-width, so the disk seats at the row's right end as the spec says; while seated the chip steps 18px left and the open stage reserves 48px on the right so nothing overlaps.
- Insert frames (not drawn in the design): hop, pop, press, seated, 80ms each; eject keeps the spec's 5 x 80ms rather than the 160ms motion unit.
- On the 96px stage the verb always takes the second line.
- A disk swap between two skills changes the sprite with no animation; the flyout shows the new bubble but no floppy.
- Attention tab: 10px, inverted (`--ps-on-st` fill, state-colour text); the ask goes 600 only in the ticket grammar, plain attention bubbles stay as before.
