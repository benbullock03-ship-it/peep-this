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

## Live run A (2026-11-16)

Throwaway worktree at e3e3710 plus a commit with an empty ticket folder and no test command, so nothing was loose. A child `claude -p /peep:ducks` was told the answers: depth 4, reject the top pick.
- Level 2: `hot: plugin 38`, `other 31`, `frontend 17`.
- Level 3: 5 cards with badge, pattern, files, problem and fix; top pick 1 (shell and edit tool lists have no single owner, scattered-concept).
- Level 4: a general-purpose sweep subagent ran and its findings joined the cards; the goose map ran (`map.mjs .`).
- Rejection: a `Considered, rejected:` line landed in `kb/backend.md`, committed as `ducks: rejected one owner for tool lists` (3f0c453, throwaway branch).
- Not seen: the depth question itself. Headless, the child loaded AskUserQuestion but used the answers given in the prompt instead of calling it.

## Live run B (2026-11-16)

Same worktree, depth 5, "None for now". The candidate rejected in run A (tool lists) did not come back; 4 new cards, top pick "Hook payloads parsed outside the Claude adapter". Bug found: "None for now" was written to `kb/backend.md` as a rejection of the top pick (f3b1eff, throwaway branch). Fixed in `depth.md`: "None for now" writes nothing.

## Owed before done

- The owner sees the "How deep?" question in an interactive `/peep:ducks` on a clean tree; headless runs cannot show it.
