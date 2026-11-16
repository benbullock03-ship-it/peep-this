---
id: AV-99
title: Walk tour splits core workflow from extras
status: closed
depends: []
---

# AV-99 — Walk tour splits core workflow from extras

**What to build:** Walk Through's tour teaches which skills make up the everyday rhythm and when to reach for the rest.

- [x] `plugin/skills/walk/references/tour.md`: a **Core** block in rhythm order (reel, qt, ticket, packup, unpack, ducks), then an **Extras** block (sneak, goose, shoot, coderot)
- [x] Each line gets a plain "use when ..." trigger, not just what the skill does
- [x] Keep the `skills.off` drop rule and the Shoot/Coderot setup notes
- [x] Add one checklist line to AV-87, AV-90 and AV-91: "add the skill to Walk's Extras block with a use-when line"

**Done when:** running `/peep:walk` on a repo with a setup prints both blocks, each skill with a use-when line, and AV-87, AV-90 and AV-91 carry the tour-line item.

## Decisions taken without a ruling

- Core keeps step numbers, renumbered 1 to 5 (why, what, build, carry on, check in) now that prove moved out; Extras are unnumbered, since they have no fixed place in the rhythm.
- The `/peep:hello` install-check line stays as the last line, outside both blocks' skill lists.
- `walk/SKILL.md` step 5 and the output block now name both blocks instead of "the rhythm block".
- Not run live: `/peep:walk` was not executed; no plugin test covers Walk.
