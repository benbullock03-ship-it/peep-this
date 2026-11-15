---
name: unpack
description: "Unpack: open a session from the Pack Up note. Reads .peep/handoff.md, checks the repo still matches it (branch, commit, uncommitted files, ticket statuses), reads back any drift, and on a yes starts the note's first command. Use when the user runs /peep:unpack, or starts a session after /peep:packup."
label: Unpack
---

# /peep:unpack

The other half of Pack Up. Carrying on is a step with a check, not "do the handoff".

## 1. Read the note

Read `.peep/handoff.md` (format: `${CLAUDE_PLUGIN_ROOT}/skills/packup/references/note.md`). Missing or empty: reply with one line, `No Pack Up note here. Try /peep:ducks to see where the repo stands.`, and stop. Not counted as a run.

## 2. Check the repo (do not print)

- Repo setup `.peep/setup.md` (`${CLAUDE_PLUGIN_ROOT}/references/formats.md` §1) for the tickets folder. No setup: assume `tickets/`. `skills.off` lists `unpack`: §1 applies.
- `git branch --show-current`, `git log --oneline -1`, `git status --short`.
- Compare with the note:
  - Branch or short hash differs: list `git log --oneline <note hash>..HEAD` (max 5) if the hash is still known.
  - Files under "Uncommitted, mine" no longer changed (committed or reverted), or new files changed that the note does not list.
  - Each ticket id the note names (in progress, parked, `Then`): read its `status` and compare with the note.
- Never fix, stage, commit, stash or restore anything here.

## 3. Read back

Format: formats §4. Groups: **Note** (in progress, left to do, first command), **Drift** (each mismatch from step 2, or "none, repo matches"), **Rulings** (owner rulings wanted, if any). Last line exactly `Reply "yes", or name what to change.`

## 4. On the answer

- "yes" or plain agreement: run the note's `First command`. If that is `/peep:unpack` itself, run the `Then` line, or the "Left to do" step if `Then` is empty. Drift that touches the command (ticket already `done`, branch changed): say so in one line and wait instead of running.
- A correction: a fresh, complete read-back, then wait again.
- Never run a push, release or deploy unless the note's first command says "approved".

## 5. Last step: run count

Bump `unpack` in `.peep/runs` (formats §1). If it was below 3, end the reply with: `Why: finish before starting, so the last session's loose ends are known before new work begins.`
