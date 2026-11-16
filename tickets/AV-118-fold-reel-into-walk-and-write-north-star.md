---
id: AV-118
title: Fold Reel into Walk and write the Peep This north star
status: open
depends: [AV-117]
---

# AV-118 — Fold Reel into Walk and write the Peep This north star

**What to build:** `/peep:reel` goes; Walk asks its questions as one round and writes `.peep/reel.md` (Question Time 2026-11-16).

- [ ] Walk gains a north star round (for, problem, hook, non-goals; comparables search only on a yes) and writes `.peep/reel.md` per formats §5 after its read-back
- [ ] Delete `plugin/skills/reel/`; remove `reel` from `PEEP_SKILLS` (`src-tauri/src/state.rs`), the floppy tray, the tour, `plugin/README.md`, spec §13 and `CONTEXT.md`
- [ ] Ducks' `no north star yet: /peep:reel` line points at `/peep:walk`
- [ ] Write this repo's `.peep/reel.md`:
  - north star: "For people new to Claude Code: a desk companion that shows which session needs you and teaches a working rhythm, one floppy at a time."
  - hook: it tells you what to do next
  - non-goals: never types into a session; no accounts or cloud
- [ ] Fix tests and fixtures that name `reel`

**Done when:** `/peep:reel` is not a skill, `/peep:walk` in a fresh repo ends with a `.peep/reel.md`, this repo has the file above, and the full run passes.
