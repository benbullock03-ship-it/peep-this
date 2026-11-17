---
id: AV-132
title: Record the AV-131 rulings
status: closed
depends: []
---

# AV-132: Record the AV-131 rulings

**What to build:** write down the decisions from the AV-131 Question Time (2026-11-17) and park the tickets it deferred.

- [x] `CLAUDE.md` Decided: the churn rule. A new skill or widget feature needs a named user problem and one real use before it gets art.
- [x] `CLAUDE.md` Decided: the v1 cut line. Rows, waiting alerts, inline approvals, bars, the signpost (AV-129), subagent and shell rows, the six Core floppies and Big Peep. Everything else comes after the release.
- [x] `CLAUDE.md` Decided: the 9/10 bar. One-step install, a signpost Jump within 5 minutes without docs, `/peep:ticket` and `/peep:packup` used unprompted by day 2, and nothing breaks across one Claude Code release.
- [x] `CLAUDE.md`: v1 ships as an unwatched open beta through the plugin marketplace. Testers prove the bar by pasting their rhythm counter. macOS comes straight after v1. Coderot stays in Extras.
- [x] Park AV-115 as `deferred` and AV-116 as `deferred`, each with `wake: top 3 beta confusion issues fixed`.
- [x] Fill AV-131 `## Answers`, putting the ticket id (AV-132 to AV-143) beside each answer, and set AV-131 to `done`.

**Done when:** `CLAUDE.md` states all the rulings above, AV-115 and AV-116 are parked with `reason` and `wake`, and AV-131 has every answer recorded.

## Checks

1. Churn rule in `CLAUDE.md` Decided: pass, `CLAUDE.md:65`.
2. v1 cut line: pass, `CLAUDE.md:66`.
3. 9/10 bar: pass, `CLAUDE.md:67`.
4. Open beta, marketplace, rhythm counter, macOS after v1, Coderot in Extras: pass, `CLAUDE.md:66` and `:68`.
5. AV-115 and AV-116 `deferred` with `reason` and `wake: top 3 beta confusion issues fixed`: pass, both frontmatters.
6. AV-131 has 15 answers each with a ticket id, status `done`: pass, grep counts 15 numbered answers naming AV-13x/AV-14x.
7. `npm run tickets` runs clean: pass.

## Decisions taken without a ruling

- The four rulings went in as bullets at the end of `CLAUDE.md` Decided; Coderot in Extras sits on the cut-line bullet.
- AV-116 was `doing` with its checklist ticked; parked as asked, its built work stays in the tree.

## Done 2026-11-17

`CLAUDE.md` Decided lines 65 to 68; AV-115 and AV-116 frontmatter; AV-131 `## Answers`.
