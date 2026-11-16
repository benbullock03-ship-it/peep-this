---
id: AV-128
title: Floppy motion and step progress
status: open
depends: [AV-124, AV-126, AV-127]
---

# AV-128 — Floppy motion and step progress

**What to build:** The seated floppy moves and shows which step its skill is on, per `design/skill-pack-3/` (Question Time 2026-11-16).

- [ ] Insert when the skill starts, the existing eject when it ends; drive light lit while it runs, blinking only while the session is `working`
- [ ] Backend: read each skill's `steps:` from the installed plugin's `SKILL.md` frontmatter and track the current step from the AV-124 signal; `Session` gains the step list and current index in the snapshot. Unknown or missing steps: floppy with no progress, never an error
- [ ] Row (16px floppy): progress as the design specifies. Open row (32px): progress plus the current step's name
- [ ] Stepped motion per the spec; reduced motion shows end states
- [ ] `cargo test` for step tracking (start, advance, skill end, unknown step); Vitest for the progress view; Shoot `?demo` both themes

**Done when:** running `/peep:ducks` shows insert, each step advancing, then eject, on the row and the open row; the owner confirms in `npm run tauri dev`.
