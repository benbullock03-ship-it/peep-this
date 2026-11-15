---
id: AV-75
title: The Reel skill - north star in .peep/reel.md, drift flagged
status: closed
depends: [AV-74]
---

# AV-75 — The Reel skill

**What to build:** `/peep:reel` establishes the project's north star and hook, so there is a reason to finish rather than abandon.

- [ ] Batched rounds (Question Time's round rules) on who it is for, the problem, the hook over alternatives, and non-goals
- [ ] Researches comparable ideas only when the owner asks; otherwise works from the answers
- [ ] Read-back (formats §4) before writing
- [ ] Writes `.peep/reel.md`: flat frontmatter `north_star` (one line), `hook`, `niche`, `for`, `non_goals`; body holds comparables when researched. Add the format to `formats.md`
- [ ] Question Time's read-back adds one line when a planned ticket does not serve the north star or hits a non-goal; never blocks
- [ ] Ducks notes drift the same way
- [ ] Why line per AV-74

**Done when:** `/peep:reel` in a test repo writes `.peep/reel.md`, and a Question Time run on an off-mission idea shows the drift line in its read-back.

## Decisions taken without a ruling

- Format added as formats.md §5 (placed before §4 in the file); §4's heading now lists Reel.
- Reel refuses with a no-setup message like Question Time, since `.peep/` lives under the repo setup.
- Drift line is `drift: <id> <why>`, under Scope in Question Time and Watch in Ducks.
- Research is a fourth question in the rounds, default "No", capped at 5 comparables.
- Why line: "a project with a reason to finish gets finished." (habit: think before code).

## Owed before close

- Live check: `/peep:reel` in a test repo writes `.peep/reel.md`; a Question Time run on an off-mission idea shows the drift line. Not run in the batch.
