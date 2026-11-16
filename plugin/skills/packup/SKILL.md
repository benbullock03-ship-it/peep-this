---
name: packup
description: "Pack Up: write a short directive note to .peep/handoff.md so a fresh session can carry on. Use when the user runs /peep:packup, says 'pack up' or 'write a handoff', is stopping, or context is filling up."
label: Pack Up
steps: [gather, ask, write, save]
---

# /peep:packup

Any time the user ends a session, not only at high context. The note opens the next session; Ducks reads it on its next run. Each heading names its step (`peep step:`, preamble).

## 1. Gather, do not print (step `gather`)

- Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, run count): the tickets folder, `build`/`test` commands and `packup.questions`. Runs without a setup (no self-setup): assume `tickets/`, no commands, `ask`.
- `git status --short`, `git branch --show-current`, `git log --oneline -1`.
- The ticket in progress this session and its status; any parked tickets (`blocked`, `deferred`) touched this session, with `reason` and `wake`.
- The last `build`/`test` run this session and its result, if any.

## 2. Ask only if the next step is unclear (step `ask`)

If the next ticket or command is obvious from the tickets and this session, or `packup.questions` is `never`, write straight away. Otherwise ask 2 or 3 questions in one batch, each with the recommended answer first, e.g. "Next: carry on AV-26 (recommended) or start AV-27?". Then write.

## 3. Write the note (step `write`)

Format and rules: `references/note.md`. Under 25 lines, directive, no background. Every path, id, hash and command copied from the tree, never recalled. When the next step is clear, the `First command` line reads exactly `First command: /peep:ducks` and the real next step goes in `Left to do` and `Then`; Ducks runs it after checking the repo. If the next step is unclear, keep the exact command there instead.

## 4. Save, copy, tell (step `save`)

1. Create `.peep/` if missing and write `.peep/handoff.md`, overwriting the last one.
2. Copy it to the clipboard with the first that works:
   - Windows: `clip < .peep/handoff.md`
   - macOS: `pbcopy < .peep/handoff.md`
   - Linux: `wl-copy < .peep/handoff.md`, else `xclip -selection clipboard < .peep/handoff.md`
3. Reply with exactly these lines, nothing else:

```
Packed up: .peep/handoff.md (<N> lines)
Clipboard: copied   (or: not copied, <reason>; open the file instead)
Safe to /clear or exit. Paste it as the first message of the next session.
next: /clear, then paste .peep/handoff.md
```

## 5. Last step: run count

Per the preamble, just before the `next` line: `Why: close the session cleanly so the next one starts where this one stopped.`

Never commit, push or stage the note. If `.peep/handoff.md` is not ignored and not tracked, leave the ignore file alone.
