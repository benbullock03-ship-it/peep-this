---
name: ducks
description: "Ducks: get your ducks in a row. Checks where the repo stands: loose tickets, parked tickets ready to wake, failing tests, stray changes, the last Pack Up note. Asks once, reads back, acts only on a yes. With no repo setup, points to /peep:walk. Use when the user runs /peep:ducks, sits down to work, or asks what to pick up next."
label: "Ducks: get your ducks in a row"
---

# /peep:ducks

The present-tense check of a repo. "F§1" to "F§4" cite the pack's shared file `${CLAUDE_PLUGIN_ROOT}/references/formats.md`. Files in `references/` beside this file are read only when their step runs.

**Nothing is written, staged or committed before the owner's yes to the read-back.**

## 0. Repo setup

Repo root: `git rev-parse --show-toplevel`. Read `<root>/.peep/setup.md` (F§1), applying its defaults.
No file, or no `prefix`: reply with exactly `peep: no repo setup here. Run /peep:walk first, it sets the repo up.` and stop; write nothing. `skills.off` lists `ducks`: F§1 applies.

## 1. Pack Up note

If `<root>/.peep/handoff.md` exists, read it before anything else. Its "not mine" files are never touched; its "Owner rulings wanted" become questions in step 4; its "Then:" line is the default next step.

## 2. Gather, cheaply

Follow `references/gather.md`: `git status`, ticket frontmatter, one `test` run, `CLAUDE.md`. It yields the loose ends: stale `doing`, `done` waiting for the owner, parked (wake met, not met, free text), failing tests, uncommitted or unrelated work, rules likely to bite. Do not print the raw gather.

## 3. Nothing loose

No loose ends and no handoff rulings: skip to the output with `clear to proceed`. No questions, no read-back.

## 4. Questions, once

One AskUserQuestion call, at most 4 questions, recommended option first with its label ending ` (recommended)`. Which questions, their order and shape: `references/questions.md`. Always include "close these done tickets?" when any ticket is `done`.
Facts are found, never asked. If no answers come back (e.g. a headless run), give the read-back with every decision left at its recommendation marked `(unconfirmed)`, then stop. Write nothing.

## 5. Read-back

Exactly per F§4, groups **Do**, **Loose**, **Watch**:

- **Do**: each action the answers chose, one per line: `wake <id> (<condition met>)`, `close <id>`, `<id> -> <status>`. "nothing" when empty.
- **Loose**: what stays as it is and why (failing test named, uncommitted files, free-text wakes kept, not-met wakes with their condition).
- **Watch**: at most 3 `CLAUDE.md` rules that bite the next ticket, each with its file and a few words; the next ticket goes here too, as `next: <id>`.

If `.peep/reel.md` exists (F§5) and the next ticket or a loose ticket does not serve its `north_star` or hits a `non_goals` entry, add one line under **Watch**: `drift: <id> <why>`. It never blocks and is not a question.

No line outside the groups except F§4's last line.

A correction gets a fresh, complete read-back. "stop" or "no": reply `peep ducks: nothing written.` and stop.

## 6. Act on yes

Only the **Do** lines, each by Edit or Write, then commit per item. Detail: `references/act.md`. Closing follows the `/peep:ticket close` procedure in `${CLAUDE_PLUGIN_ROOT}/skills/ticket/references/close.md`; the yes to this read-back is the owner's yes for the ids it names. Never touch another session's uncommitted files.

## Output

The final reply is these lines and nothing else:

```
peep ducks: <n> loose ends, <n> acted on
- <action taken, e.g. "AV-12 woke (AV-09 closed)" or "AV-07 closed (abc1234)">   one per line, omitted when none
loose: <short list of what stays open> | none
next: /peep:ticket <id> | clear to proceed
```

`next` is the handoff's "Then:" if still valid, else the lowest-numbered `open` ticket whose `depends` are all `closed`. Nothing loose: `peep ducks: 0 loose ends, 0 acted on` then `next: clear to proceed`.

## Last step: run count

Bump `ducks` in `.peep/runs` (F§1). If it was below 3, add a final line: `Why: finish what is loose before you start something new.`
