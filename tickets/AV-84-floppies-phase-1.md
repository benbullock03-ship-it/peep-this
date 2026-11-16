---
id: AV-84
title: Floppies for Unpack, Walk Through and The Reel
status: doing
depends: [AV-75, AV-76, AV-77, AV-79]
attended: true
---

# AV-84 — Phase 1 floppies

**What to build:** The three new skills get floppies in the tray, on the plank and in first run, from the AV-79 sprites.

- [x] Add `unpack`, `walk`, `reel` to the floppy list (`src/tray.ts`) with commands, colours and hover copy from the design
- [x] Backend maps the new commands to skills so the inserted floppy shows (AV-64)
- [x] Update vitest and the tray overflow for nine floppies; Shoot both themes

**Done when:** the owner sees each new floppy inserted while its skill runs and Jumps it from the tray.

## Owed before done

- Owner, in `npm run tauri dev`: run `/peep:unpack`, `/peep:walk` and `/peep:reel` in a session and see each floppy seat on the plank; open the row, press `+3` and Jump each one.

## Decisions taken without a ruling

- New sprites appended to `src/assets/floppy/floppies-{dark,light}.png` as columns 10 to 12 (sheet now 208 px wide); `design/skill-pack/sprites/` left unchanged.
- Tray shows the six spec floppies, then a `+3` cell opening a 5-column grid of the other three under the row (FLOPPY-SPEC §02); the grid is its own roving tab stop.
- Tray cells narrowed from 44 px to 40 px wide (height stays 44) so six disks and `+3` fit the 312 px tray row.
- Bubble verbs: unpacking, walking through, reeling.
- First-run disk box gets one extra line: the three disks at 32 px, "Unpack, Walk Through, The Reel sit behind +3 in the tray."
- Colours come from the sprites only; the `--fl-*` tokens are left for AV-83, which first uses one.
