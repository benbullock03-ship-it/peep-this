---
id: AV-119
title: Fold Scrub into Shoot
status: closed
depends: []
---

# AV-119 — Fold Scrub into Shoot

**What to build:** `/peep:scrub` goes; its hallmark pass becomes `/peep:shoot --scrub` (Question Time 2026-11-16).

- [x] `--scrub` on Shoot: after the normal report, run the grep pass (`scan.mjs`) and the eye pass on the shots already taken, then the numbered findings list and the same reply rule (numbers, `debt <n>`, `wrong <n>`, `none`)
- [x] Move `scan.mjs`, its fixtures and `eye.md`/`fix.md` under `plugin/skills/shoot/`; the catalogue stays at `plugin/references/ai-tells.md`
- [x] Shoot's `SKILL.md` stays under 80 lines: the scrub pass lives in a reference file
- [x] Delete `plugin/skills/scrub/`; remove `scrub` from `PEEP_SKILLS`, the floppy tray, the tour, `plugin/README.md`, spec §13 and `CONTEXT.md`
- [x] Fix tests and fixtures that name `scrub`

**Done when:** `/peep:scrub` is not a skill, `/peep:shoot --scrub` on a UI diff prints the Shoot report then the numbered findings, the scan tests pass from their new home, and the full run passes.

## Done 2026-11-16

`plugin/skills/scrub/` gone; scan, fixtures, eye/fix and `references/scrub.md` under `plugin/skills/shoot/`; `PEEP_SKILLS` has 11 entries. build, vitest (369 plus 216 for scan/ai-tells) and cargo test pass. A live `/peep:shoot --scrub` run is owed to the owner.

## Decisions taken without a ruling

- Kept `.peep/scrub.md` and inline `scrub-ignore:` names unchanged (no rename asked; `.peep/` untouchable here).
- Scrub pass runs only under Shoot's `url` gate; the "without a url" inferred mode in `eye.md` is kept only for a hand-run scan.
- Removed the Scrubadubdub CONTEXT entry and folded one sentence into Shoot's entry; spec §13 never named scrub, so unchanged.
- Floppy sprite column 14 left unused.
