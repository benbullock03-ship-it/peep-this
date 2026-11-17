---
id: AV-122
title: One Question Time mode
status: closed
depends: []
---

# AV-122 — One Question Time mode

**What to build:** Question Time always asks in AskUserQuestion rounds; the switch to a numbered text list at run 3 goes (Question Time 2026-11-16).

- [x] Remove List mode from `qt/SKILL.md` §3 and `references/rounds.md`; round size is `qt.round`, default 4
- [x] No AskUserQuestion tool (headless): fall back to the numbered list, once, and say so
- [x] Walk and Sneak, which cite these rules, need no change beyond the reference

**Done when:** with `qt: 5` in `.peep/runs` and no `qt.round`, `/peep:qt <idea>` still asks in dialog rounds of 4.

## Done 2026-11-16

`plugin/skills/qt/SKILL.md` §3 and `references/rounds.md`: AskUserQuestion rounds of `qt.round` (default 4) whatever the run count; List mode gone, numbered list only when the tool is missing. Live check with `qt: 5` still owed to the owner.
