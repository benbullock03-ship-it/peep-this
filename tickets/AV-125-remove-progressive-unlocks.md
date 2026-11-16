---
id: AV-125
title: Remove progressive unlocks
status: open
depends: []
---

# AV-125 — Remove progressive unlocks

**What to build:** Core and Extras only, no rules; supersedes AV-108 (Question Time 2026-11-16).

- [ ] Remove `src/unlocks.ts`, its tests, and the locked floppy state from the tray; Extras always show and work
- [ ] Remove the Unlocks section and locked block from `plugin/skills/walk/references/tour.md`; every Extra keeps its one-line "use when"
- [ ] Shoot still sits in Extras when the setup has no `url` (it cannot run there); that is the one rule left
- [ ] Remove the `demo` repo's locked Extras fixture if nothing else uses it
- [ ] AV-108: add a note `Superseded by AV-125 on 2026-11-16` and leave its status for the owner
- [ ] Earned cosmetics (AV-115) are untouched: accessories are still earned, skills are never locked
- [ ] Update spec and `CONTEXT.md`

**Done when:** a fresh repo's tray shows every Extra enabled, the tour prints no `Locked:` block, and the full run passes.
