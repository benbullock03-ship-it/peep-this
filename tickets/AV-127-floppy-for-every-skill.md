---
id: AV-127
title: Floppy for every skill
status: open
depends: [AV-118, AV-119]
---

# AV-127 — Floppy for every skill

**What to build:** Every Peep skill invocation shows its floppy seated on that session's plank (Question Time 2026-11-16).

- [ ] Ship the floppies already drawn in `design/skill-pack-2/sprites/` for walk, sneak, goose, bigpeep and tweet; all eleven remaining skills have a floppy, both themes, with disabled variants
- [ ] Tray order: Core (walk, qt, ticket, shoot, packup, ducks), then Extras (sneak, goose, coderot, bigpeep)
- [ ] Tweet is inserted-only: its floppy seats on the plank when Tweet runs (hook or command) but has no tray cell and no Jump
- [ ] Each floppy's hover-card line comes from the tour's "use when" text
- [ ] Fix `plugin/README.md`'s table, which still says "none yet" for art that exists
- [ ] Vitest: every name in `PEEP_SKILLS` maps to a sprite; Shoot `?demo` in both themes

**Done when:** running each of the eleven skills in a session seats its floppy on that row, Tweet shows no tray cell, and the owner confirms in `npm run tauri dev`.
