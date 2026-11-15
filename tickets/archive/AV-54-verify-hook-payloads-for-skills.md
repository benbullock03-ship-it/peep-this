---
id: AV-54
title: Verify hook payloads carry skill commands, file paths and exit codes
status: closed
depends: []
attended: false
---

# AV-54 — Verify hook payloads for skills, clash and worry

**What to build:** Prove, against a live session, the three signals the widget work assumes. Nothing in the widget phase starts until this passes or the plan changes.

- [x] `UserPromptSubmit`: does `prompt` contain a plugin slash command verbatim (`/peep:ticket 34`), or is it expanded to the skill body first? Also check a skill Claude invokes itself via the Skill tool (does a `PreToolUse` with `tool_name: Skill` arrive, and with which input?)
- [x] `PreToolUse`/`PostToolUse` for `Edit`, `Write`, `MultiEdit`, `NotebookEdit`: absolute file path present
- [x] `PostToolUse` for `Bash`: command text and exit code (or an error flag) present
- [x] Add anonymised payloads to `src-tauri/tests/fixtures/`
- [x] Note any gap in the ticket and in the widget tickets it blocks (AV-64, AV-69, AV-71)

**Done when:** each signal is confirmed or a fallback is written into the blocked tickets.

## Findings (live, headless `claude -p` with logging hooks, 2026-11-15)

- **Slash command:** confirmed. `UserPromptSubmit.prompt` is verbatim, `"/caveman:caveman-help extra-arg-42"`. A Claude-invoked skill fires `PreToolUse`/`PostToolUse` with `tool_name: "Skill"`, `tool_input: { skill: "caveman:caveman-help", args: "probe" }`; `PostToolUse.tool_response` is `{ success, commandName }`.
- **File paths:** confirmed, absolute. `tool_input.file_path` for `Edit`/`Write` (MultiEdit takes the same field per its schema; not exercised live), `tool_input.notebook_path` for `NotebookEdit`. `PostToolUse.tool_response.filePath` echoes it.
- **Bash result:** partly. Success: `PostToolUse` with `tool_input.command` and `tool_response.{stdout, stderr, interrupted}`, no exit code. Failure: `PostToolUseFailure` with `tool_input.command`, `error: "Exit code N\n<stderr>"`, `is_interrupt`. No numeric exit code field; the event name is the failure flag. The installer already registers `PostToolUseFailure` (`ingest/hooks.rs`).
- Gaps and fallbacks written into AV-64, AV-69 and AV-71.

## Decisions taken without a ruling

- Exercised `/caveman:caveman-help` as the plugin slash command since `peep` doesn't exist yet; plugin namespacing is the same.
- Fixtures added without Rust tests; AV-64, AV-69 and AV-71 own the parsers that will load them.
