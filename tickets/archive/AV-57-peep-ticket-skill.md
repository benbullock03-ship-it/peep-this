---
id: AV-57
title: Ticket skill with the six-status lifecycle and area notes
status: closed
depends: [AV-56]
attended: false
---

# AV-57 — `/peep:ticket`

**What to build:** Port the global `ticket` skill into the pack, made generic through the repo setup.

- [ ] `/peep:ticket <id>` works one ticket end to end: read it and its depends, set `doing`, build, run the repo setup's checks, commit only its hunks, set `done`
- [ ] `/peep:ticket close <id>`: only on the owner's yes; sets `closed`, `git mv` to the archive, edits the ticket's area note (rewrite, don't append), regenerates the index
- [ ] `/peep:ticket park <id>`: sets `blocked` or `deferred`; refuses without a reason and a wake-up condition
- [ ] Reads the one relevant area note before building, never the whole archive
- [ ] No repo setup found: stop and point to `/peep:ducks`

**Done when:** a test repo runs open, doing, done, closed and park through the skill, and the area note and index update correctly.

## Decisions taken without a ruling

- Depends must be `closed` (formats §2), not `done`; an unclosed depend auto-parks the ticket `blocked`, `wake: ticket:<dep>`, and the run stops.
- Working a parked ticket unparks it when its wake is checkably met (`ticket:` closed, `date:` passed) or the owner says so; otherwise it prints reason and wake and stops.
- `close` counts the owner typing or pasting `/peep:ticket close <id>` as the yes; if Claude got there itself, or the status is not `done`, it asks first.
- `close` and `park` each make their own commit (`<id>: close`, `<id>: park (<status>)`), separate from the work commit.
- Area note on close: the area most of the ticket's committed files fall in; tie goes to the first area in setup order.
- `park` without a status picks `blocked` for a `ticket:` wake, else `deferred`.
- Waking other tickets when one closes is left to Ducks, per spec §13.1.
- A bare number expands to `<prefix>-<n>`.

## Test evidence 2026-11-15

Scratch repo (`.peep/setup.md` prefix TT, node build/test/index scripts, `kb/maths.md`, TT-01..03), headless `claude --plugin-dir plugin -p ... --permission-mode acceptEdits` with an `--allowedTools` list for git, node and mkdir:
- No setup: printed the one-line `/peep:ducks` pointer.
- `TT-01`: open -> done; `sub` added, test passing, items ticked, `## Done`, index regenerated, one commit with only its files.
- `TT-02` (depends on TT-01 at `done`): auto-parked `blocked`, `reason`/`wake: ticket:TT-01`, committed.
- `park TT-03` with no reason: refused, changed nothing. With reason and a date: `deferred`, `wake: date:2026-12-01`.
- `close TT-01`: `closed`, `git mv` to `tickets/archive/` (rename in the commit), `kb/maths.md` rewritten (stale "no subtraction" bullet gone, `last touched: TT-01`), index regenerated.
- `TT-02` again after TT-01 closed: unparked (`reason`/`wake` removed), built, blocked -> done, committed; clean tree.

## Owed before done

- `doing` is set mid-run; a one-shot headless run only shows the end state. Owner: run `/peep:ticket <id>` interactively and check the ticket reads `status: doing` while it builds.
- `close` from a path Claude reached itself (should ask "Close <id>? Reply yes.") is untested; it needs an interactive session.
- Installed-plugin path (`claude plugin install peep@peep-this`) not exercised; only `--plugin-dir`.
