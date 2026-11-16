---
id: AV-118
title: Fold Reel into Walk and write the Peep This north star
status: closed
depends: [AV-117]
---

# AV-118 — Fold Reel into Walk and write the Peep This north star

**What to build:** `/peep:reel` goes; Walk asks its questions as one round and writes `.peep/reel.md` (Question Time 2026-11-16).

- [x] Walk gains a north star round (for, problem, hook, non-goals; comparables search only on a yes) and writes `.peep/reel.md` per formats §5 after its read-back
- [ ] Delete `plugin/skills/reel/`; remove `reel` from `PEEP_SKILLS` (`src-tauri/src/state.rs`), the floppy tray, the tour, `plugin/README.md`, spec §13 and `CONTEXT.md`
- [x] Ducks' `no north star yet: /peep:reel` line points at `/peep:walk`
- [x] Write this repo's `.peep/reel.md`:
  - north star: "For people new to Claude Code: a desk companion that shows which session needs you and teaches a working rhythm, one floppy at a time."
  - hook: it tells you what to do next
  - non-goals: never types into a session; no accounts or cloud
- [x] Fix tests and fixtures that name `reel`

**Done when:** `/peep:reel` is not a skill, `/peep:walk` in a fresh repo ends with a `.peep/reel.md`, this repo has the file above, and the full run passes.

## Decisions taken without a ruling

- Round C is a fourth AskUserQuestion: For, Problem, Hook, Non-goals. Comparables are the Hook question's "Look up comparables first" option, so a search happens only on that pick (`walk/references/northstar.md`).
- Walk's read-back regroups as **Repo**, **Areas**, **Habits**, **North star** to stay within F§4's four groups.
- The tour's Core drops its "why" line and renumbers; Walk's `next` follows the tour (`/peep:qt` with no open tickets, else `/peep:ducks`).
- `skills.off` examples and Walk's "Skills off" options swap Reel for Sneak.
- The arc line's "No north star yet" button jumps to `/peep:walk` and takes the walk floppy's label colour (`--fl-walk` from `design/skill-pack-2/tokens.css`) in place of `--fl-reel`.
- Floppy art column 12 (reel) stays in the sprite sheet unused; the art is the owner's.
- `.peep/reel.md` stays uncommitted like `.peep/setup.md`.

## Done 2026-11-16

- `plugin/skills/reel/` deleted; `PEEP_SKILLS` has 12 names; tray, bubble, learned, tour and README drop `reel` (7f263be). Spec §13 never named Reel.
- Walk's round C and write: `plugin/skills/walk/references/northstar.md`. This repo's `.peep/reel.md` exists (uncommitted).
- Full run passes: vitest 369, cargo 424+17.
- `CONTEXT.md` Reel entry is fixed in another session's uncommitted edit; it lands when that session commits.
- Owner confirmed done 2026-11-16 without the live `/peep:walk` run or a shoot of the arc line's "Walk Through" button.
