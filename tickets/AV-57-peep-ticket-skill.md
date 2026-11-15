---
id: AV-57
title: Ticket skill with the six-status lifecycle and area notes
status: open
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
