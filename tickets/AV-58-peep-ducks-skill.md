---
id: AV-58
title: Ducks skill - repo check, first-run setup, batch close, wake parked tickets
status: open
depends: [AV-56, AV-57]
attended: false
---

# AV-58 — `/peep:ducks` ("Get your ducks in a row")

**What to build:** The present-tense check of a repo.

- [ ] First run with no repo setup: a batched round of questions (prefix, commands, areas, all with guessed recommended answers), read-back, then write the setup
- [ ] Reads `.peep/handoff.md` from the last Pack Up first, if present
- [ ] Every run, gathered cheaply (git status, the ticket index, one test run): `doing` tickets with no recent commits, `done` tickets waiting for the owner, parked tickets, failing tests, uncommitted or unrelated work, rules in `CLAUDE.md` likely to bite the next change
- [ ] Wakes parked tickets whose wake-up condition is met (referenced ticket closed, date passed); free-text conditions are listed for the owner, not judged silently
- [ ] Batched questions about anything the owner must decide, including "close these done tickets?"
- [ ] Read-back, then a tidy list or "clear to proceed"; nothing written before the yes

**Done when:** on a test repo with each kind of loose end, Ducks finds all of them, asks once, and acts only on the confirmed items.
