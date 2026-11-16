---
id: AV-104
title: Ducks depth ladder, levels 1 to 4
status: blocked
reason: waiting on AV-103
wake: ticket:AV-103
depends: [AV-103]
---

# AV-104 — Ducks depth ladder, levels 1 to 4

**What to build:** Pocock-style architecture review reached through Ducks, without charging every sit-down for it. Only when nothing is loose does Ducks ask "how deep? 1-5". Levels stack by cost; each option states its rough cost before running.

- [ ] Nothing loose and no handoff rulings: one AskUserQuestion, "How deep?", level 1 recommended
- [ ] 1: today's Ducks, `clear to proceed`
- [ ] 2: plus hot spots, the areas (setup `area.<name>`) most touched in the last ~20 commits
- [ ] 3: plus candidate cards in text from the hot spots: deletion test as the only filter ("would deleting it concentrate complexity?"), strength badge (Strong, Worth exploring, Speculative), files, one-line problem and fix; one top pick; stop and ask which to explore
- [ ] 4: plus a fresh-context subagent sweep (shallow modules, a concept spread over many files, leaks across seams, untested code) and the goose basic map
- [ ] Candidates already rejected in the area note are skipped; a new rejection writes one line `Considered, rejected: <what> - <why>` there; an ADR only when the reason is hard to reverse or surprising
- [ ] No `.peep/reel.md`: one Watch line `no north star yet: /peep:reel`, at most once a day
- [ ] Never edits code; acting on a candidate means drafting a ticket on yes

**Done when:** on a clean tree Ducks asks the depth, levels 2 to 4 each produce their output on this repo, a rejection lands in the area note and is not re-suggested on the next run.
