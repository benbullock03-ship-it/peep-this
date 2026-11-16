---
name: ducks
description: "Ducks: check where the repo stands and carry on from the last Pack Up note, acting only on a yes. Use when the user runs /peep:ducks."
disable-model-invocation: true
label: "Ducks: get your ducks in a row"
---

# /peep:ducks

The present-tense check of a repo. "F§1" to "F§4" cite the pack's shared file `${CLAUDE_PLUGIN_ROOT}/references/formats.md`. Files in `references/` beside this file are read only when their step runs. **Nothing is written, staged or committed before the owner's yes to the read-back.**

## 0. Repo setup

Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count). Its setup yes is separate from the read-back yes below.

## 1. Pack Up note

If `<root>/.peep/handoff.md` exists, read it before anything else (format: `${CLAUDE_PLUGIN_ROOT}/skills/packup/references/note.md`). Its "not mine" files are never touched; its "Owner rulings wanted" become questions in step 4; its `Then:` line is the first **Do** line (step 5). Check the repo against it, without printing or fixing anything: `references/drift.md`. Each mismatch goes under **Loose** as `drift: <what>`.

## 2. Gather, cheaply

Follow `references/gather.md`: `git status`, ticket frontmatter, one `test` run, `CLAUDE.md`. It yields the loose ends, `done` always first with a count: `done` waiting for the owner, stale `doing`, parked (wake met, not met, free text), failing tests, uncommitted or unrelated work, rules likely to bite. Do not print the raw gather.

## 3. Nothing loose

No loose ends and no Pack Up note: no question, no read-back; straight to the output with `next: clear to proceed`. Hot spots and pattern cards are Goose's (`/peep:goose health`), not Ducks'. Ducks never edits code.

## 4. Questions, once

One AskUserQuestion call, at most 4 questions, recommended option first with its label ending ` (recommended)`. Which questions, their order and shape: `references/questions.md`. Always include "close these done tickets?" when any ticket is `done`.
Facts are found, never asked. If no answers come back (e.g. a headless run), give the read-back with every decision left at its recommendation marked `(unconfirmed)`, then stop. Write nothing.

## 5. Read-back

Exactly per F§4, groups **Do**, **Loose**, **Watch**:

- **Do**: each action the answers chose, one per line: `run <the note's Then>` first when a Pack Up note has one (dropped when drift touches it, e.g. that ticket is already `done` or the branch changed; say why under **Loose**), then `close <n> done: <id>, <id>`, then `wake <id> (<condition met>)`, `<id> -> <status>`. "nothing" when empty. The count is stated here, before anything moves; the owner can reply `leave <ids>` to take ids out of the batch.
- **Loose**: what stays as it is and why (failing test named, uncommitted files, free-text wakes kept, not-met wakes with their condition).
- **Watch**: at most 3 `CLAUDE.md` rules that bite the next ticket, each with its file and a few words; the next ticket goes here too, as `next: <id>`.

If `.peep/reel.md` exists (F§5) and the next ticket or a loose ticket does not serve its `north_star` or hits a `non_goals` entry, add one line under **Watch**: `drift: <id> <why>`. It never blocks and is not a question.
No `.peep/reel.md` and the `ducks.reel` date in `.peep/runs` is not today: one **Watch** line `no north star yet: /peep:walk` (also in the output's `loose:` when there is no read-back), then set `ducks.reel: <YYYY-MM-DD>`.

No line outside the groups except F§4's last line. A correction gets a fresh, complete read-back. "stop" or "no": reply `peep ducks: nothing written.` and stop.

## 6. Act on yes

Only the **Do** lines. A `run` line goes first: run that command before the other Do lines; never a push, release or deploy unless the note says "approved". The rest each by Edit or Write, then commit per item. Detail: `references/act.md`. Closing follows the `/peep:ticket close` procedure in `${CLAUDE_PLUGIN_ROOT}/skills/ticket/references/close.md`; the yes to this read-back is the owner's yes for the ids it names. Never touch another session's uncommitted files.

## Output

The final reply is these lines and nothing else:

```
peep ducks: <n> loose ends, <n> acted on
- <action taken, e.g. "AV-12 woke (AV-09 closed)" or "AV-07 closed (abc1234)">   one per line, omitted when none
loose: <short list of what stays open, done left out first, then stale doing> | none
next: /peep:ticket <id> | clear to proceed
```

`next` is the handoff's "Then:" if still valid, else the lowest-numbered `open` ticket whose `depends` are all `closed`. Nothing loose, three lines: `peep ducks: 0 loose ends, 0 acted on`, `deeper look: /peep:goose health`, `next: clear to proceed`. A no-north-star nudge (step 5) adds `loose: no north star yet: /peep:walk` before `next`.

## Last step: run count

Per the preamble: `Why: finish what is loose before you start something new.`
