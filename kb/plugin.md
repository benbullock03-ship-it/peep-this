# plugin

## How it works now

- `plugin/` is the `peep` Claude Code plugin; each skill is `plugin/skills/<name>/SKILL.md` with step detail in its `references/`.
- `/peep:walk` owns first run: two batched rounds (repo, then how the owner works), a read-back, then `.peep/setup.md` (F§1) with only non-default preference keys (`qt.round`, `packup.questions`, `shoot.widths`, `skills.off`), a tour in rhythm order and a suggested first step, usually `/peep:reel`. Every other skill with no setup points to `/peep:walk`.
- Shared formats live once in `plugin/references/formats.md` (F§1 to F§5); skills cite the section instead of restating it.
- Every skill's last step bumps its line in `.peep/runs` and, while the old count is below 3, ends with one `Why:` line tied to a habit: think before code, finish before starting, prove it works.
- `/peep:reel` asks batched rounds on who it is for, the problem, the hook and non-goals, reads back, then writes `.peep/reel.md` (F§5). Research of comparables runs only if asked, at most 5.
- Question Time (under Scope) and Ducks (under Watch) read `.peep/reel.md` only to add a `drift: <id> <why>` line for a ticket off the north star or on a non-goal.
- Ducks lists stale `doing` tickets first: ticket file unchanged for the setup's `stale.days` (default 3, file mtime), asked as finish, park with a reason, or close. The widget's backend worries the repo's peeps on the same rule (`tickets.rs` `Repo.stale`).
- `/peep:sneak` asks Question Time-style design rounds, reads back, builds 2 or 3 self-contained variants in `.peep/sneak/<slug>/{a,b,c}/`, serves them side by side with `skills/sneak/scripts/serve.mjs` (127.0.0.1, random port), and moves the pick to `design/<slug>/`, deleting the rest. It adds `.peep/sneak/` to `.gitignore` itself and never touches the app.
- `/peep:goose` runs `skills/goose/scripts/map.mjs`: ts/js and Rust imports (or `--graphify graphify-out/graph.json`), files grouped by setup areas, live sessions placed by cwd, written to `.peep/graph/graph.json` (`edges` `[from, to]`) and a standalone `index.html` with `peep.js` inlined from `skills/goose/assets/`. Query instructions sit in `kb/INDEX.md`.
- `/peep:packup` writes `.peep/handoff.md` (template in `skills/packup/references/note.md`) with `First command: /peep:unpack` when the next step is clear. `/peep:unpack` checks branch, commit, uncommitted files and named ticket statuses against it, reads back the drift, and on yes runs the first command.

## Watch out

- Runs that stop early (no setup, bad argument) are not counted.
- `skills.off` is stated once in F§1; each skill carries a one-line cite. Walk can never be switched off; the tour lists The Reel unless `skills.off` names it.
- The Why line is the only addition allowed after a skill's fixed output format.
- `.peep/runs` is per machine and not committed; skills mention a missing `.gitignore` entry rather than editing it.
- Stale tickets and drift never block new work.
- Drift never blocks and is never a question. No `.peep/reel.md`, no drift line.
- In `formats.md`, §5 sits above §4 in the file.
- Unpack runs the note's `Then:` line when the first command is `/peep:unpack` itself, to avoid a loop. Drift touching the command (ticket already `done`, branch changed) makes it wait rather than run on yes. No note: one line, not counted.

## Why it's like this

- New users learn the reason behind each step, then stop seeing it once the habit is familiar.
- The Reel gives a project a reason to finish rather than abandon.
- Unpack makes carrying on a real step that catches what changed between sessions, not "do the handoff".
- Ducks no longer writes the setup; one skill owning first run keeps the setup questions in one place.

last touched: AV-92, 2026-11-15
