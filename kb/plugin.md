# plugin

## How it works now

- `plugin/` is the `peep` Claude Code plugin; each skill is `plugin/skills/<name>/SKILL.md` with step detail in its `references/`.
- Shared formats live once in `plugin/references/formats.md` (F§1 to F§5); skills cite the section instead of restating it.
- Every skill's last step bumps its line in `.peep/runs` and, while the old count is below 3, ends with one `Why:` line tied to a habit: think before code, finish before starting, prove it works.
- `/peep:reel` asks batched rounds on who it is for, the problem, the hook and non-goals, reads back, then writes `.peep/reel.md` (F§5). Research of comparables runs only if asked, at most 5.
- Question Time (under Scope) and Ducks (under Watch) read `.peep/reel.md` only to add a `drift: <id> <why>` line for a ticket off the north star or on a non-goal.

## Watch out

- Runs that stop early (no setup, bad argument) are not counted.
- The Why line is the only addition allowed after a skill's fixed output format.
- `.peep/runs` is per machine and not committed; skills mention a missing `.gitignore` entry rather than editing it.
- Drift never blocks and is never a question. No `.peep/reel.md`, no drift line.
- In `formats.md`, §5 sits above §4 in the file.

## Why it's like this

- New users learn the reason behind each step, then stop seeing it once the habit is familiar.
- The Reel gives a project a reason to finish rather than abandon.

last touched: AV-75, 2026-11-15
