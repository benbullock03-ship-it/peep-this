# plugin

## How it works now

- `plugin/` is the `peep` Claude Code plugin; each skill is `plugin/skills/<name>/SKILL.md` with step detail in its `references/`.
- Shared formats live once in `plugin/references/formats.md` (F§1 to F§5); skills cite the section instead of restating it.
- Every skill's last step bumps its line in `.peep/runs` and, while the old count is below 3, ends with one `Why:` line tied to a habit: think before code, finish before starting, prove it works.
- `/peep:reel` asks batched rounds on who it is for, the problem, the hook and non-goals, reads back, then writes `.peep/reel.md` (F§5). Research of comparables runs only if asked, at most 5.
- Question Time (under Scope) and Ducks (under Watch) read `.peep/reel.md` only to add a `drift: <id> <why>` line for a ticket off the north star or on a non-goal.
- `/peep:packup` writes `.peep/handoff.md` (template in `skills/packup/references/note.md`) with `First command: /peep:unpack` when the next step is clear. `/peep:unpack` checks branch, commit, uncommitted files and named ticket statuses against it, reads back the drift, and on yes runs the first command.

## Watch out

- Runs that stop early (no setup, bad argument) are not counted.
- The Why line is the only addition allowed after a skill's fixed output format.
- `.peep/runs` is per machine and not committed; skills mention a missing `.gitignore` entry rather than editing it.
- Drift never blocks and is never a question. No `.peep/reel.md`, no drift line.
- In `formats.md`, §5 sits above §4 in the file.
- Unpack runs the note's `Then:` line when the first command is `/peep:unpack` itself, to avoid a loop. Drift touching the command (ticket already `done`, branch changed) makes it wait rather than run on yes. No note: one line, not counted.

## Why it's like this

- New users learn the reason behind each step, then stop seeing it once the habit is familiar.
- The Reel gives a project a reason to finish rather than abandon.
- Unpack makes carrying on a real step that catches what changed between sessions, not "do the handoff".

last touched: AV-76, 2026-11-15
