---
id: AV-124
title: next line and steps frontmatter in every skill
status: doing
depends: [AV-117, AV-118, AV-119, AV-120, AV-121, AV-122, AV-123]
---

# AV-124 — next line and steps frontmatter in every skill

**What to build:** Two things the widget reads from every remaining Peep skill (walk, qt, ticket, shoot, packup, ducks, sneak, goose, coderot, bigpeep, tweet) (Question Time 2026-11-16).

- [x] Every skill's fixed output ends with one line `next: <command>` or `next: clear to proceed`. Rule goes in the shared preamble (AV-117)
- [x] Every `SKILL.md` frontmatter gains `steps:` as a flat list of short step names in order, e.g. `steps: [look, ask, read-back, write]`; at most 6, each at most 10 characters
- [x] The skill says which step it is on in a form a hook can see: decide and document the signal (recommended: a one-line `peep step: <name>` at the start of each step)
- [x] Sneak ends `next: /peep:qt <slug>` so the pick becomes tickets; Sneak asks its own rounds only when no Question Time answers exist in the session
- [x] Big Peep's brief passes step names to little peeps

**Done when:** each of the eleven skills has `steps:` in frontmatter and a `next:` last line in its output format, and one run of `/peep:ducks` shows the step signal for each step.

## Decisions taken without a ruling

- Step signal is the recommended text line `peep step: <name>` (regex `^peep step: ([a-z-]+)$`), not a tool call: hooks only see tool calls, so a tool-based signal would cost a call (and maybe a prompt) per step; AV-128 reads the line from the transcript tail. Documented in formats §8 and the preamble.
- `next:` is the reply's very last line: the run-count Why line now goes just before it (preamble; ticket and packup lose their "fifth/fourth line" wording).
- Each SKILL.md heading names its step as `(step `<name>`)`, or `(still `<name>`)` where two sections share a step; ticket and tweet map sub-commands to steps in one line instead.
- Lines that stop a run early (switched off, nothing written, bad argument) and replies that wait for the owner (read-back, offer, scrub findings) carry no `next:`.
- A `next:` value that is not a command or `clear to proceed` is a note for the owner (work.md's `none unblocked; ...`, Big Peep's `none`); formats §8 says so.
- Shoot: housekeeping moved before the report (the report already said "Shots: deleted"); `next` is `/peep:shoot <change>` while a `fail` or `can't tell` stands. With `--scrub`, `next` ends fix.md's output (`/peep:sneak <screen>` when a design choice is needed).
- Ticket `next`: `done` -> `/peep:ticket close <id>`, `doing` -> `/peep:ticket <id>`, parked by a dep -> work.md's next-ticket walk (moved from `owed` to `next`), close and park -> `/peep:ducks`.
- Packup `next: /clear, then paste .peep/handoff.md`. Big Peep keeps its report's `next` last and drops the Pack Up lines' own `next`.
- Goose `next: /peep:qt <top pick>` for health with a top pick, else `clear to proceed`. Coderot: `/peep:ticket <first id>` after writing tickets, else `clear to proceed`. Tweet: `clear to proceed`, or `/peep:tweet reply <n> <answer>` from the list.
- Sneak's skip-the-rounds rule now names only Question Time; the `/peep:shoot --scrub` source was dropped, as the ticket asks.
