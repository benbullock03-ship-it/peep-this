---
id: AV-122
title: One Question Time mode
status: open
depends: []
---

# AV-122 — One Question Time mode

**What to build:** Question Time always asks in AskUserQuestion rounds; the switch to a numbered text list at run 3 goes (Question Time 2026-11-16).

- [ ] Remove List mode from `qt/SKILL.md` §3 and `references/rounds.md`; round size is `qt.round`, default 4
- [ ] No AskUserQuestion tool (headless): fall back to the numbered list, once, and say so
- [ ] Walk and Sneak, which cite these rules, need no change beyond the reference

**Done when:** with `qt: 5` in `.peep/runs` and no `qt.round`, `/peep:qt <idea>` still asks in dialog rounds of 4.
