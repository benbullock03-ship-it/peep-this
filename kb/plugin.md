# plugin

## How it works now

- `plugin/` is the `peep` Claude Code plugin; each skill is `plugin/skills/<name>/SKILL.md` with step detail in its `references/`.
- `/peep:walk` owns first run: two batched rounds (repo, then how the owner works), a read-back, then `.peep/setup.md` (F§1) with only non-default preference keys (`qt.round`, `packup.questions`, `shoot.widths`, `skills.off`), a tour (a numbered **Core** block in rhythm order: reel, qt, ticket, packup, unpack, ducks; then an unnumbered **Extras** block, each skill with a "use when" line) and a suggested first step, usually `/peep:reel`. Every other skill with no setup points to `/peep:walk`.
- Shared formats live once in `plugin/references/formats.md` (F§1 to F§7); skills cite the section instead of restating it.
- The AI hallmark catalogue is `plugin/references/ai-tells.md` (82 entries, slug IDs, P0 to P2 by who notices); its grep rules live in `ai-tells/rules.mjs` with `scan(text)`, and `rules.test.mjs` holds a bad and a good snippet per rule and keeps the md and `RULES` in step. Per-repo exceptions are `.peep/scrub.md` (F§6: `earned` permanent, `debt` may only shrink) plus inline `scrub-ignore: <id> <reason>`.
- `/peep:scrub` (Scrubadubdub) runs `skills/scrub/scripts/scan.mjs`: catalogue grep rules over changed lines since `main` (falls back to `master`, then `HEAD`) plus untracked files, or `--full`; exceptions applied; one finding per ID per file, scored by severity plus the other IDs in that file. Then an eye pass (`references/eye.md`) that always covers spacing and text, on Shoot shots or `inferred` without a `url`. The numbered report phrases alternatives in the repo's own tokens. On the reply (`references/fix.md`) it fixes only the approved numbers, sends design choices to `/peep:sneak`, writes debt on `debt <n>`, and on `wrong <n>` offers an earned line or names a rule bug.
- Every skill's last step bumps its line in `.peep/runs` and, while the old count is below 3, ends with one `Why:` line tied to a habit: think before code, finish before starting, prove it works.
- `/peep:reel` asks batched rounds on who it is for, the problem, the hook and non-goals, reads back, then writes `.peep/reel.md` (F§5). Research of comparables runs only if asked, at most 5.
- Question Time (under Scope) and Ducks (under Watch) read `.peep/reel.md` only to add a `drift: <id> <why>` line for a ticket off the north star or on a non-goal.
- Ducks lists `done` tickets first as one item with a count, and the read-back's **Do** line `close <n> done: <ids>` closes them all on one yes, one `<id>: close` commit each (`leave <ids>` drops some). Stale `doing` comes second: ticket file unchanged for the setup's `stale.days` (default 3, file mtime), asked as finish, park with a reason, or close. The widget's backend worries the repo's peeps on the same rule (`tickets.rs` `Repo.stale`).
- `/peep:sneak` asks Question Time-style design rounds, reads back, builds 2 or 3 self-contained variants in `.peep/sneak/<slug>/{a,b,c}/`, serves them side by side with `skills/sneak/scripts/serve.mjs` (127.0.0.1, random port), and moves the pick to `design/<slug>/`, deleting the rest. It adds `.peep/sneak/` to `.gitignore` itself and never touches the app.
- `/peep:goose` runs `skills/goose/scripts/map.mjs`: ts/js and Rust imports (or `--graphify graphify-out/graph.json`), files grouped by setup areas, written to `.peep/graph/graph.json` (`edges` `[from, to]`) and a standalone `index.html` with `peep.js` inlined from `skills/goose/assets/`. Query instructions sit in `kb/INDEX.md`. A live session's peep stands on each area its Edit and Write calls touched in the last 30 minutes, read from the tail of its transcript (`~/.claude/projects/<cwd with non-alphanumerics as ->/<sessionId>.jsonl`); with none it falls back to its cwd's area or a `repo root` card.
- `/peep:bigpeep` is the global `/batch` rebuilt on the repo setup: little peeps follow `work.md` but never stage, commit or set status; Big Peep sets `doing`, reviews the diff, commits one per ticket. A little peep quiet 10 minutes gets one SendMessage status ask (`references/watch.md`), never a stop, and answers by SendMessage to `main`. An in-batch dep that reaches `done` earlier in the run counts as met.
- `/peep:tweet` posts to `.peep/tweets.md` (F§7), never staged or committed; a live ping via SendMessage is only a nudge to read the board. `/peep:ticket` reads the board before building (work.md 1.5) and folds answered tweets into area notes on close (close.md 4).
- `/peep:shoot` checks a desktop app's webview over CDP (`shoot/references/webview.md`: debug port, `Runtime.evaluate`, the app's own commands, OS screenshots for native chrome) and applies `references/stable.md` before every fact and shot; a verdict that changes on rerun is `flaky`.
- A blocked `/peep:ticket <id>` still parks it on the first direct dep not `closed`, then walks the depends tree depth-first (`ticket/references/work.md`) and names the next step in `owed`: `next: /peep:ticket <dep>` for the first open dep whose own depends are closed, or `/peep:ticket close <dep>` for a `done` one. `doing` deps are skipped; with none unblocked it names the nearest wake.
- `/peep:packup` writes `.peep/handoff.md` (template in `skills/packup/references/note.md`) with `First command: /peep:unpack` when the next step is clear. `/peep:unpack` checks branch, commit, uncommitted files and named ticket statuses against it, reads back the drift, and on yes runs the first command.

## Watch out

- Runs that stop early (no setup, bad argument) are not counted.
- `skills.off` is stated once in F§1; each skill carries a one-line cite. Walk can never be switched off; the tour lists The Reel unless `skills.off` names it.
- The Why line is the only addition allowed after a skill's fixed output format.
- `.peep/runs` is per machine and not committed; skills mention a missing `.gitignore` entry rather than editing it.
- Stale tickets and drift never block new work.
- Drift never blocks and is never a question. No `.peep/reel.md`, no drift line.
- A new skill adds itself to Walk's Extras block in `walk/references/tour.md` with a use-when line.
- A grep rule's good snippet must hit no rule at all; a false positive is a bug. The scanner always skips the pack's own catalogue and `skills/scrub/fixtures/` (they hit every rule by design); repo reference folders like `design/` are passed as `--skip`.
- Scrubadubdub never writes an earned line (those are the owner's), never stages or commits, and never picks a design choice; Sneak does that. Anything declared in the repo's DESIGN.md is never flagged; this repo's seed earns only the AV-44 shell choices.
- A live ping only lands when the peer runs in the same permission mode; otherwise its harness holds the message for approval and the board carries it alone.
- Goose's graphify input works from a directed or undirected build, but graphify's own `cache_root` sets how `source_file` is made relative: build from the repo root or the paths won't match the setup areas. Its callers differ a little from the import map's (it misses `mod x;` links).
- Live checks run by Claude never send keystrokes to a window: prompts have defaults (a trust dialog's is "No, exit"). Child `claude` sessions need every `CLAUDE*` variable cleared.
- Unpack runs the note's `Then:` line when the first command is `/peep:unpack` itself, to avoid a loop. Drift touching the command (ticket already `done`, branch changed) makes it wait rather than run on yes. No note: one line, not counted.

## Why it's like this

- New users learn the reason behind each step, then stop seeing it once the habit is familiar.
- The Reel gives a project a reason to finish rather than abandon.
- Unpack makes carrying on a real step that catches what changed between sessions, not "do the handoff".
- Ducks no longer writes the setup; one skill owning first run keeps the setup questions in one place.

last touched: AV-92, 2026-11-16
