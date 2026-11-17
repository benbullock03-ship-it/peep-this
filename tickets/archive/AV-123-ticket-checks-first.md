---
id: AV-123
title: Ticket checks first
status: closed
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

1. work.md §2 writes numbered `open:` checks into the ticket's `## Checks` in the status edit, then prints them as `Checks:`: pass, `plugin/skills/ticket/references/work.md` §2.
2. work.md §5.2 answers those checks in place `pass`/`fail`/`owed to the owner` with evidence, `(added)` for late ones: pass, work.md §5.2.
3. `done` only with no `fail`; owed under `## Owed before done`: pass, work.md §5.3.
4. Four-line report unchanged, checks in the ticket: pass, `plugin/skills/ticket/SKILL.md` §3.
5. Ticket file ends with answered checks citing evidence: pass, this section.
6. Live run shows checks before any edit: pass (Big Peep, 2026-11-17, third run). Throwaway `claude -p "/peep:ticket LC-03"` in a scratch repo: the `## Checks` edit with three `open:` checks is stream line 23, the `Checks:` reply line 25, the first code edit line 26; final ticket answered each in place with `pass` and evidence. Stream: scratchpad `lc123-stream3.jsonl`.
   - Runs one and two failed (lines 46 and 41 edited code with no `Checks:` anywhere): a reply-only rule was read and skipped, so §2 now puts the checks in the ticket edit.

## Done 2026-11-17

`plugin/skills/ticket/references/work.md` §2 and §5.2; `plugin/skills/bigpeep/references/brief.md` sends little peeps through §2's checks list; live run in check 6.

## Decisions taken without a ruling

- The checks are written into the ticket in the same edit as `status: doing`, before any other file change, and printed after; a message-only rule was skipped in two live runs.
- `formats.md` and `docs/spec.md` left untouched: neither documents the report sections beyond the "Done when" line.
