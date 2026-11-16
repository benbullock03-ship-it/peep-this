---
id: AV-123
title: Ticket checks first
status: doing
depends: []
---

# AV-123 — Ticket checks first

**What to build:** `/peep:ticket` borrows Shoot's rule: the checks are written before the work and answered after it (Question Time 2026-11-16).

- [x] Before building, turn the ticket's "Done when" into a numbered list of pass/fail checks and post it in a message of its own
- [x] At the end, answer exactly those checks `pass`, `fail` or `owed to the owner`, each with its evidence (test name, command output, shot path). A check added later is marked `(added)`
- [x] `done` only when no check is `fail`; `owed` checks go under `## Owed before done` as today
- [x] The four-line report is unchanged; the answered checks go in the ticket under `## Checks`

**Done when:** working a ticket shows the numbered checks before any edit and the ticket file ends with them answered, each citing evidence.

## Checks

1. work.md §3 tells the worker to post numbered checks (`Checks:` message) before any edit: pass, `plugin/skills/ticket/references/work.md` §3 first bullet.
2. work.md §5.2 answers exactly those checks `pass`/`fail`/`owed to the owner` with evidence, `(added)` for late ones: pass, work.md §5.2.
3. `done` only with no `fail`; owed under `## Owed before done`: pass, work.md §5.3.
4. Four-line report unchanged, checks in the ticket: pass, `plugin/skills/ticket/SKILL.md` §3.
5. Ticket file ends with answered checks citing evidence: pass, this section.
6. Checks posted in a message before any edit during this run: owed to the owner (the rule did not exist while this ticket was built, so the list above was written after the fact; watch a later `/peep:ticket` run).

## Owed before done

- Check 6 is a live behaviour: owner or a later run must watch `/peep:ticket` post `Checks:` before editing. Not provable from skill text.

## Decisions taken without a ruling

- Checks are posted at the start of §3 (Build), after the Start step, so they precede every edit but follow the status change.
- `formats.md` and `docs/spec.md` left untouched: neither documents the report sections beyond the "Done when" line.
