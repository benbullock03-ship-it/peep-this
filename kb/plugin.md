# plugin

## How it works now

- `plugin/` is the `peep` Claude Code plugin; each skill is `plugin/skills/<name>/SKILL.md` with step detail in its `references/`.
- Shared formats live once in `plugin/references/formats.md` (F§1 to F§5); skills cite the section instead of restating it.
- Every skill's last step bumps its line in `.peep/runs` and, while the old count is below 3, ends with one `Why:` line tied to a habit: think before code, finish before starting, prove it works.

## Watch out

- Runs that stop early (no setup, bad argument) are not counted.
- The Why line is the only addition allowed after a skill's fixed output format.
- `.peep/runs` is per machine and not committed; skills mention a missing `.gitignore` entry rather than editing it.

## Why it's like this

- New users learn the reason behind each step, then stop seeing it once the habit is familiar.

last touched: AV-74, 2026-11-15
