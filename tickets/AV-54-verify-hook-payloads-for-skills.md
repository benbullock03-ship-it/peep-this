---
id: AV-54
title: Verify hook payloads carry skill commands, file paths and exit codes
status: open
depends: []
attended: false
---

# AV-54 — Verify hook payloads for skills, clash and worry

**What to build:** Prove, against a live session, the three signals the widget work assumes. Nothing in the widget phase starts until this passes or the plan changes.

- [ ] `UserPromptSubmit`: does `prompt` contain a plugin slash command verbatim (`/peep:ticket 34`), or is it expanded to the skill body first? Also check a skill Claude invokes itself via the Skill tool (does a `PreToolUse` with `tool_name: Skill` arrive, and with which input?)
- [ ] `PreToolUse`/`PostToolUse` for `Edit`, `Write`, `MultiEdit`, `NotebookEdit`: absolute file path present
- [ ] `PostToolUse` for `Bash`: command text and exit code (or an error flag) present
- [ ] Add anonymised payloads to `src-tauri/tests/fixtures/`
- [ ] Note any gap in the ticket and in the widget tickets it blocks (AV-64, AV-69, AV-71)

**Done when:** each signal is confirmed or a fallback is written into the blocked tickets.
