---
name: reel
description: "Reel: pin down why this project exists. Asks batched rounds on who it is for, the problem, the hook over alternatives and the non-goals, reads it back, then writes .peep/reel.md with the north star. Researches comparable ideas only when asked. Use when the user runs /peep:reel, or starts a project with no clear reason to finish it."
label: "Reel: the north star"
---

# /peep:reel

Gives the project a reason to finish. The project is the text after `/peep:reel`; with none, take it from the repo (`CONTEXT.md`, `CLAUDE.md`, README) and say so in one line.
"formats" below is the pack's shared file `${CLAUDE_PLUGIN_ROOT}/references/formats.md`.

## 1. Read the repo setup

Read `.peep/setup.md` at the repo root (formats §1). If it is missing, reply with exactly this and stop:

`peep: no repo setup here. Run /peep:walk first, so Reel knows where .peep/ lives.`

`skills.off` lists `reel`: formats §1 applies.

## 2. Look before asking

Read `CONTEXT.md` if present, the README, and `.peep/reel.md` if it exists (a rerun starts from its values, and each question names the current one).
Facts are yours to find, never the user's. Decisions are the user's.

## 3. Rounds

Ask with the AskUserQuestion tool, 4 questions per round, rules as Question Time's (`${CLAUDE_PLUGIN_ROOT}/skills/qt/SKILL.md` §3, shape and wording in `${CLAUDE_PLUGIN_ROOT}/skills/qt/references/rounds.md`): recommended option first ending ` (recommended)`, each question standalone, loose ends before new ground, a "Keep going" question while the frontier is not empty.

Cover, in this order of dependency:

- **For:** who uses it, and in what moment.
- **Problem:** what hurts for them today.
- **Hook:** the one thing that makes them pick this over the alternatives they have now (name the alternatives; offer them as options).
- **Non-goals:** what it will not do, even when asked.
- **Research:** "Look up comparable ideas?" Default "No (recommended)". Search only on a yes.

## 4. Comparables (only on a yes)

Search the web for a few comparable products. Keep what bears on the hook: name, link, one line on how it differs. At most 5. Fold any that undercut the hook into one question to the owner before the read-back.

## 5. Read-back

Exactly per formats §4. Groups **North star** (one line: for whom, what, why this), **Hook**, **Not**. Add **Comparables** only when researched.
"yes" goes to step 6; a correction gets a fresh, complete read-back; "stop" replies `peep: read-back kept, nothing written.` and stops.

## 6. Write `.peep/reel.md`

Format: formats §5. Write by Edit or Write, replacing any earlier file. Nothing else: no code, no ticket edits.

## Output

```
peep reel: north star written
north star: <one line>
```

## Last step: run count

Bump `reel` in `.peep/runs` (formats §1). If it was below 3, add a final line: `Why: a project with a reason to finish gets finished.`
