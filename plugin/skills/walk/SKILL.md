---
name: walk
description: "Walk Through: tour and customising that asks about the repo, how you work and what the project is for, then writes .peep/setup.md and .peep/reel.md. Use when the user runs /peep:walk."
disable-model-invocation: true
label: "Walk Through"
---

# /peep:walk

Optional: any skill sets up a repo with no setup itself (preamble). Walk asks the fuller rounds, tailors the pack, pins down the project's north star, shows the order the skills go in. "F§1" and "F§4" cite `${CLAUDE_PLUGIN_ROOT}/references/formats.md`. Files in `references/` beside this file are read only when their step runs.

**Nothing is written before the owner's yes to the read-back.** Walk is never switched off by `skills.off`.

## 1. Look

Repo root: `git rev-parse --show-toplevel`. Read `<root>/.peep/setup.md` if it exists (F§1): a rerun starts from its values and each question names the current one. Then guess the answers from the repo, as `references/setup.md` step 1 says. Facts are found, never asked.

## 2. Rounds

Ask with AskUserQuestion, batched, recommended option first ending ` (recommended)`, each question standalone, no "Keep going" question. Round A is the repo, round B is how the owner works: `references/setup.md`. Round C is the north star: `references/northstar.md`. Shape and wording rules: `${CLAUDE_PLUGIN_ROOT}/skills/qt/references/rounds.md`.

## 3. Read-back

Exactly per F§4, one read-back covering all three rounds: groups and rules in `references/setup.md` §4, plus **North star** from `references/northstar.md` §3. "yes": steps 4 and 5. A correction: a fresh, complete read-back, then wait. "stop" or "no": reply `peep walk: nothing written.` and stop.

## 4. Write the setup

`.peep/setup.md` per `references/setup.md` step 5: the repo keys plus only the preference keys that differ from their defaults (`qt.round`, `packup.questions`, `shoot.widths`, `skills.off`; F§1). Edit or Write only. Do not stage or commit it.

## 5. Write the north star

`.peep/reel.md` per `references/northstar.md` §4.

## 6. Tour

Read `.peep/runs` and the tickets and archive folders, then print the Core and Extras blocks in `references/tour.md` with locked Extras staged per its Unlocks, then the next step. Do not run any skill.

## Output

```
peep walk: setup written (.peep/setup.md new, uncommitted; .peep/reel.md new, uncommitted)
north star: <one line>
<the Core and Extras blocks, then any locked Extras>
next: <command>
```

`next` is the tour's next step (`references/tour.md`).

## Last step: run count

Per `${CLAUDE_PLUGIN_ROOT}/references/preamble.md`: `Why: a minute of setup now saves every later skill from guessing.`
