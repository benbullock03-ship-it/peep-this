---
id: AV-89
title: Floppies for Sneak Peep and Scrubadubdub
status: done
depends: [AV-85, AV-87, AV-88]
attended: true
---

# AV-89 — Phase 2 floppies

**What to build:** The two design skills get floppies, from the AV-88 sprites.

- [x] Add `sneak` and `scrub` to the floppy list (`src/tray.ts`) with commands, colours and hover copy
- [x] Backend maps the commands to skills so the inserted floppy shows
- [x] Update vitest and tray overflow; Shoot both themes

**Done when:** the owner sees both floppies inserted while their skills run and Jumps them from the tray.

## Done 2026-11-16

- `src/tray.ts` TRAY gains Sneak Peep and Scrubadubdub with SPEC §07 hover copy; `src/bubble.ts` FLOPPIES maps them to sheet columns 13 and 14 (the label colours are in the AV-88 sprites, so no CSS was needed); `state.rs` `PEEP_SKILLS` lists both, so the inserted floppy seats.
- Tests: `tray.test.ts` (order, `+7` overflow, unlock rules now covered by TRAY alone) and cargo `phase_1_and_2_skills_seat_their_floppy`. `npm run build`, vitest 361 and `cargo test` 410 pass.
- Shot in the `?demo` page, dark and light: `+7` opens walk, reel, sneak, scrub, bigpeep, tweet, goose in a 5 + 2 grid inside the tray; sneak shows the magenta eye, scrub the pale-blue suds; the disk box line reads "... sit behind +7".
- Live check (`tauri dev`, CDP on 9333, two throwaway `claude -p` sessions in scratch git repos): `get_snapshot` gave `skill=sneak` and `skill=scrub`. The screenshot showed the magenta and pale-blue floppies inserted on the planks, with bubbles "· mocking up" and "· scrubbing".
- Jump: a real click on each disk under `+7` copied the command. The hint read "Copied /peep:sneak" and then "Copied /peep:scrub", and the clipboard read back `/peep:scrub`. This ran in the `?demo` page, not the widget: a concurrent session's half-edited `state.rs` (AV-109 `tip`) broke the dev rebuild before the in-widget Jump. Terminal focus on Jump is AV-67's unchanged path.

## Decisions taken without a ruling

- Tray order follows SPEC §07: sneak and scrub sit after reel and before bigpeep.
- Bubble verbs: "mocking up" for sneak, "scrubbing" for scrub.
