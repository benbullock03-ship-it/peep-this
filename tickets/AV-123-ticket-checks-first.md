---
id: AV-123
title: Ticket checks first
status: open
depends: []
---

# AV-123 — Ticket checks first

**What to build:** `/peep:ticket` borrows Shoot's rule: the checks are written before the work and answered after it (Question Time 2026-11-16).

- [ ] Before building, turn the ticket's "Done when" into a numbered list of pass/fail checks and post it in a message of its own
- [ ] At the end, answer exactly those checks `pass`, `fail` or `owed to the owner`, each with its evidence (test name, command output, shot path). A check added later is marked `(added)`
- [ ] `done` only when no check is `fail`; `owed` checks go under `## Owed before done` as today
- [ ] The four-line report is unchanged; the answered checks go in the ticket under `## Checks`

**Done when:** working a ticket shows the numbered checks before any edit and the ticket file ends with them answered, each citing evidence.
