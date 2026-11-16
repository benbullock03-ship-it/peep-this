---
id: AV-107
title: Coderot hot-spot default and adaptive Question Time rounds
status: done
depends: []
---

# AV-107 — Coderot hot-spot default and adaptive Question Time rounds

**What to build:** Two small skill changes from the reflective pass.

- [x] Coderot with no argument targets the area most touched in the last ~20 commits and says which, instead of refusing; cap stays 20
- [x] Question Time: `qt` run count below 3, rounds of 4 by AskUserQuestion as now; from 3, the whole frontier as one numbered text list with a recommendation each, reply "agree all" or named overrides
- [x] A `qt.round` setup value still wins over both

**Done when:** `/peep:coderot` with no argument picks and names an area on this repo, and Question Time's `rounds.md` describes both modes keyed on the run count.

## Decisions taken without a ruling

- Hot spot counts files (not commits) from `git log -20 --name-only` per area prefix; ties go to setup order; no area matched still asks for one.
- List mode drops the "Keep going" item; the list is the whole frontier and loose ends come back as a fresh list.

## Done 2026-11-16

- `plugin/skills/coderot/SKILL.md` §1 Target: the hot-spot rule. On this repo at 891425f it picks `frontend` (38 files, against plugin 37 and backend 11). Checked by running the rule's `git log` count, not a full Coderot run.
- `plugin/skills/qt/references/rounds.md` "Two modes" and "List shape"; `qt/SKILL.md` §3 picks the mode from the run count, and `qt.round` wins.
