---
id: AV-32
title: While-you-were-away digest
status: todo
depends: [AV-26]
attended: false
---

# AV-32 — While-you-were-away digest

**What to build:** Stub. On return from being away, one card that says what happened, ordered by what needs the user first (spec §11.9). Built from events only; no model call.

- [ ] Away: screen locked, or no input for 5 min (setting). Return: unlock or first input
- [ ] Card groups sessions: blocked on you, failed, finished, still running. Per session: time taken, files changed (count and names), commands run (count), last assistant line, approvals released to the terminal while away
- [ ] Each entry focuses its session; blocked entries open the inbox
- [ ] No card when nothing changed. Dismissed by Esc or any decision
- [ ] The per-session activity log behind it is in memory, capped, and holds paths and counts only
- [ ] Tests: grouping, ordering, empty case

**Done when:** the owner locked the screen during real work, came back, and the card matched what had happened.
