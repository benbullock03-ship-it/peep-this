---
id: AV-124
title: next line and steps frontmatter in every skill
status: open
depends: [AV-117, AV-118, AV-119, AV-120, AV-121, AV-122, AV-123]
---

# AV-124 — next line and steps frontmatter in every skill

**What to build:** Two things the widget reads from every remaining Peep skill (walk, qt, ticket, shoot, packup, ducks, sneak, goose, coderot, bigpeep, tweet) (Question Time 2026-11-16).

- [ ] Every skill's fixed output ends with one line `next: <command>` or `next: clear to proceed`. Rule goes in the shared preamble (AV-117)
- [ ] Every `SKILL.md` frontmatter gains `steps:` as a flat list of short step names in order, e.g. `steps: [look, ask, read-back, write]`; at most 6, each at most 10 characters
- [ ] The skill says which step it is on in a form a hook can see: decide and document the signal (recommended: a one-line `peep step: <name>` at the start of each step)
- [ ] Sneak ends `next: /peep:qt <slug>` so the pick becomes tickets; Sneak asks its own rounds only when no Question Time answers exist in the session
- [ ] Big Peep's brief passes step names to little peeps

**Done when:** each of the eleven skills has `steps:` in frontmatter and a `next:` last line in its output format, and one run of `/peep:ducks` shows the step signal for each step.
