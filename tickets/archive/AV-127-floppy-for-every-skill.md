---
id: AV-127
title: Floppy for every skill
status: closed
depends: [AV-118, AV-119]
---

# AV-127 — Floppy for every skill

**What to build:** Every Peep skill invocation shows its floppy seated on that session's plank (Question Time 2026-11-16).

- [x] Ship the floppies already drawn in `design/skill-pack-2/sprites/` for walk, sneak, goose, bigpeep and tweet; all eleven remaining skills have a floppy, both themes, with disabled variants
- [x] Tray order: Core (walk, qt, ticket, shoot, packup, ducks), then Extras (sneak, goose, coderot, bigpeep)
- [x] Tweet is inserted-only: its floppy seats on the plank when Tweet runs (hook or command) but has no tray cell and no Jump
- [x] Each floppy's hover-card line comes from the tour's "use when" text
- [x] Fix `plugin/README.md`'s table, which still says "none yet" for art that exists
- [x] Vitest: every name in `PEEP_SKILLS` maps to a sprite; Shoot `?demo` in both themes

**Done when:** running each of the eleven skills in a session seats its floppy on that row, Tweet shows no tray cell, and the owner confirms in `npm run tauri dev`.

## Checks (live, 2026-11-16, `tauri dev` over CDP)

1. pass: each of the eleven skills seats its floppy. A headless `claude -p "/peep:<skill>" --plugin-dir plugin` in a scratch repo, for each skill, polled `get_snapshot` and the row DOM. Every one seated with the right `--col` and verb: walk 11, qt 1, ticket 2, shoot 3, packup 4, ducks 0, sneak 13, goose 17, coderot 5, bigpeep 15, tweet 16.
2. pass: the open row's tray holds Walk, Question Time, Ticket, Shoot, Pack Up and Ducks, then `+4` (Sneak, Goose, Coderot, Big Peep). The tray HTML has no `tweet`. Evidence: `.playwright-mcp/live-tray.png`.

## Decisions taken without a ruling

- Status set to `done` from the live run above. The visual pass is the owner's close.
- AV-119 is `done`, not `closed`; treated as satisfied because Big Peep dispatched this ticket with scrub already removed and committed.
- The sprite sheets in `src/assets/floppy/` already matched `design/skill-pack-2/sprites/` byte for byte; no files were copied.
- The tour has no Walk line, so Walk's hover card keeps SPEC §06's "Use the first time, or when the pack feels off." The tour was not edited, because Walk prints that tour after it runs.
- The first-run disk box numbers the tour's five (Question Time, Ticket, Shoot, Pack Up, Ducks) and sets Walk apart, in place of the old Coderot line; Coderot now sits with the Extras behind +4.
- The `?demo` row tweets-16 now runs Tweet, so its plank shows the tweet floppy for Shoot.
- The Rust test `phase_1_and_2_skills_seat_their_floppy` became `every_peep_skill_seats_its_floppy`, which loops over all of `PEEP_SKILLS`.
