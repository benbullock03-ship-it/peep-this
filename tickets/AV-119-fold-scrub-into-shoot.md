---
id: AV-119
title: Fold Scrub into Shoot
status: open
depends: []
---

# AV-119 — Fold Scrub into Shoot

**What to build:** `/peep:scrub` goes; its hallmark pass becomes `/peep:shoot --scrub` (Question Time 2026-11-16).

- [ ] `--scrub` on Shoot: after the normal report, run the grep pass (`scan.mjs`) and the eye pass on the shots already taken, then the numbered findings list and the same reply rule (numbers, `debt <n>`, `wrong <n>`, `none`)
- [ ] Move `scan.mjs`, its fixtures and `eye.md`/`fix.md` under `plugin/skills/shoot/`; the catalogue stays at `plugin/references/ai-tells.md`
- [ ] Shoot's `SKILL.md` stays under 80 lines: the scrub pass lives in a reference file
- [ ] Delete `plugin/skills/scrub/`; remove `scrub` from `PEEP_SKILLS`, the floppy tray, the tour, `plugin/README.md`, spec §13 and `CONTEXT.md`
- [ ] Fix tests and fixtures that name `scrub`

**Done when:** `/peep:scrub` is not a skill, `/peep:shoot --scrub` on a UI diff prints the Shoot report then the numbered findings, the scan tests pass from their new home, and the full run passes.
