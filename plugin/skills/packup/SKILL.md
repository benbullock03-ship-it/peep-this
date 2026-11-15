---
name: packup
description: "Pack Up: carry this session forward into a new one. Writes a short directive note to .peep/handoff.md, copies it to the clipboard, then says it is safe to /clear or exit. Use when the user runs /peep:packup, says 'pack up', 'write a handoff', 'I'm stopping for the day', or context is filling up."
label: Pack Up
---

# /peep:packup

Any time the user ends a session, not only at high context. The note opens the next session; Ducks reads it on its next run.

## 1. Gather (do not print)

- Repo setup `.peep/setup.md` (`${CLAUDE_PLUGIN_ROOT}/references/formats.md` §1) for the tickets folder and `build`/`test` commands. No setup: assume `tickets/` and no commands.
- `git status --short`, `git branch --show-current`, `git log --oneline -1`.
- The ticket in progress this session and its status; any parked tickets (`blocked`, `deferred`) touched this session, with `reason` and `wake`.
- The last `build`/`test` run this session and its result, if any.

## 2. Ask only if the next step is unclear

If the next ticket or command is obvious from the tickets and this session, write straight away. Otherwise ask 2 or 3 questions in one batch, each with the recommended answer first, e.g. "Next: carry on AV-26 (recommended) or start AV-27?". Then write.

## 3. Write the note

Format and rules: `references/note.md`. Under 25 lines, directive, no background. Every path, id, hash and command copied from the tree, never recalled. When the next step is clear, the `First command` line reads exactly `First command: /peep:unpack` and the real next step goes in `Left to do` and `Then`; Unpack runs it after checking the repo. If the next step is unclear, keep the exact command there instead.

## 4. Save, copy, tell

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
```

## 5. Last step: run count

Bump `packup` in `.peep/runs` (formats §1). If it was below 3, add a fourth reply line: `Why: close the session cleanly so the next one starts where this one stopped.`

Never commit, push or stage the note. If `.peep/handoff.md` is not ignored and not tracked, leave the ignore file alone.
