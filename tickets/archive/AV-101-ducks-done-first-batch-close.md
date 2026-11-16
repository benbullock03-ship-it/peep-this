---
id: AV-101
title: Ducks lists done tickets first and offers batch close
status: closed
depends: []
---

# AV-101 — Ducks lists done tickets first and offers batch close

**What to build:** Done tickets stop piling up in `tickets/`. Archiving stays `closed`-only; Ducks makes closing them a single yes.

- [x] `plugin/skills/ducks/references/gather.md`: `done` tickets are the first item in the check, with a count
- [x] `act.md`: one yes closes them all, following `/peep:ticket close` (status, `git mv` to the archive, area note, index)
- [x] The owner can name ids to leave out of the batch
- [x] The read-back states the count before anything moves

**Done when:** running `/peep:ducks` in this repo lists AV-72 (and any other `done`) first, and a yes leaves no `done` tickets in `tickets/`.

## Done

- `gather.md` lists `done` first as one item, `<n> done: <ids>`; stale `doing` moves to second.
- `act.md` closes every id on the `close <n> done: <ids>` line, one `<id>: close` commit each via `close.md` step 2; re-checks each is still `done` before moving.
- `SKILL.md`: gather order, the batch **Do** line with its count, `leave <ids>` to drop ids; the `Close` question's multi-select and Other subset (`questions.md`, unchanged) also leave ids out.
- Dry-run of gather on 2026-11-15: `2 done: AV-72, AV-80`; read-back **Do** line `close 2 done: AV-72, AV-80`.

## Owed before done

- The real `/peep:ducks` run and the owner's yes that empties `done` from `tickets/`.

## Decisions taken without a ruling

- `done` now outranks stale `doing` in gather, read-back and the `loose:` line; the question order in `questions.md` was left as is (out of scope), and SKILL.md still forces the `Close` question whenever a ticket is `done`.
- One commit per closed ticket, not one batch commit, per the global one-commit-per-ticket rule and `close.md` step 5.
