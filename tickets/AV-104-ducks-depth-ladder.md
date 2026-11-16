---
id: AV-104
title: Ducks depth ladder, levels 1 to 4
status: doing
depends: [AV-103]
---

# AV-104 — Ducks depth ladder, levels 1 to 4

**What to build:** Pocock-style architecture review reached through Ducks, without charging every sit-down for it. Only when nothing is loose does Ducks ask "how deep? 1-5". Levels stack by cost; each option states its rough cost before running.

- [x] Nothing loose and no handoff rulings: one AskUserQuestion, "How deep?", level 1 recommended
- [x] 1: today's Ducks, `clear to proceed`
- [x] 2: plus hot spots, the areas (setup `area.<name>`) most touched in the last ~20 commits
- [x] 3: plus candidate cards in text from the hot spots: deletion test as the only filter ("would deleting it concentrate complexity?"), strength badge (Strong, Worth exploring, Speculative), files, one-line problem and fix; one top pick; stop and ask which to explore
- [x] 4: plus a fresh-context subagent sweep (shallow modules, a concept spread over many files, leaks across seams, untested code) and the goose basic map
- [x] Candidates already rejected in the area note are skipped; a new rejection writes one line `Considered, rejected: <what> - <why>` there; an ADR only when the reason is hard to reverse or surprising
- [x] No `.peep/reel.md`: one Watch line `no north star yet: /peep:reel`, at most once a day
- [x] Never edits code; acting on a candidate means drafting a ticket on yes

**Done when:** on a clean tree Ducks asks the depth, levels 2 to 4 each produce their output on this repo, a rejection lands in the area note and is not re-suggested on the next run.

## Decisions taken without a ruling

- Woken before AV-103 closed: the owner said "finish 105" on 2026-11-16, which needs this ticket first.
- AskUserQuestion allows 4 options, so it offers levels 1 to 4 and level 4's description says "or reply 5 for the page".
- The reel nudge date lives in `.peep/runs` as `ducks.reel: <date>`; F§1 now allows `<skill>.<key>` state lines.
- Rejections go under the area note's `## Why it's like this` and are committed as `ducks: rejected <what>`.

## Owed before done

- Live run in a fresh session on a clean tree: Ducks asks the depth; levels 2, 3 and 4 produce their output; a rejection lands in the area note and is skipped on the next run. Level 2 on this repo at f993559 gives frontend 38, plugin 37, backend 11 (the hot-spot count was run by hand).
