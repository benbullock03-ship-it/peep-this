---
id: AV-100
title: Ticket skill offers the next unblocked dep
status: done
depends: []
---

# AV-100 — Ticket skill offers the next unblocked dep

**What to build:** When a dep blocks `/peep:ticket <id>`, the skill still parks it (`work.md` §1.2) and also points to the next ticket to run.

- [x] Walk the depends tree for the first dep that is `open` with all of its own depends `closed`
- [x] Report line `owed` names it: `next: /peep:ticket <dep>`
- [x] A dep that is `done` but not `closed`: suggest `/peep:ticket close <dep>`
- [x] No unblocked dep (all parked): say which wake-up condition is nearest
- [x] Update `plugin/skills/ticket/references/work.md` only; one ticket per run stays

**Done when:** running `/peep:ticket` on a ticket whose dep chain has an open, unblocked ticket parks it and names that ticket as next.

## Decisions taken without a ruling

- Walk is depth-first in `depends` order; the first `done` or open-unblocked dep wins, whichever comes first.
- A `doing` dep is skipped and not walked into (another session has it); if only `doing` deps remain, `next` names one.
- Blocked/deferred deps are walked into, so a parked dep's open prerequisite is still offered.
- "Nearest wake": earliest `date:`, else first `ticket:` in walk order, else first free text.
- The park itself is unchanged: `reason`/`wake` name the first direct dep not `closed`, not the next ticket.

## Worked example (hand walk, 2026-11-15)

- AV-87 depends [AV-74, AV-86]: AV-74 archived (closed), skip; AV-86 open, depends [] -> `next: /peep:ticket AV-86`. Park AV-87 `wake: ticket:AV-86`.
- AV-97 -> AV-96 open with AV-95 not closed, walk in -> AV-95 open, depends [] -> `next: /peep:ticket AV-95`. Park wake `ticket:AV-96`.
- AV-89 depends [AV-85, AV-87, AV-88]: AV-85 doing, skip; AV-87 open with AV-86 not closed, walk in -> AV-74 closed, AV-86 open unblocked -> `next: /peep:ticket AV-86`. Park wake `ticket:AV-85`.
