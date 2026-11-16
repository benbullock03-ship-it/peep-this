---
id: AV-108
title: Progressive unlocks in the Walk tour and floppies
status: open
depends: [AV-103]
---

# AV-108 — Progressive unlocks in the Walk tour and floppies

**What to build:** A beginner sees the core rhythm first; Extras appear as they earn them. Skills stay callable at any time; only the tour and floppies stage them.

- [ ] Unlocks from `.peep/runs` and tickets: Big Peep after 5 `ticket` runs; Coderot after the first ticket closed with a test command in setup; Goose after 3 tickets closed; Sneak and Scrub after the first ticket closed that touched a UI area; Shoot in Core when setup has a `url`
- [ ] Walk tour prints locked Extras greyed with their unlock, e.g. `/peep:bigpeep  unlocks after 5 tickets`
- [ ] Widget floppy tray shows locked floppies dimmed with the unlock on hover; Jump still works
- [ ] Big Peep's first-run Why: watch the diffs, not the reports
- [ ] Vitest for the unlock rules

**Done when:** a fresh repo's tour shows locked Extras with their unlocks, this repo's tour shows them unlocked, and the floppy tray matches; the owner confirms the tray.
