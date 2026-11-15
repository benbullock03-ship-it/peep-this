---
id: AV-71
title: Clash when two sessions edit the same file
status: open
depends: [AV-54, AV-63, AV-69, AV-70]
attended: true
---

# AV-71 — Clash

Owner ruling 2026-11-15: ADR 0002 stands over FLOPPY-SPEC ruling 2. Clash is an ambient mood: startled emote, file name and bolt tag on both bubbles, dotted bracket only when the rows are already adjacent; no pin, re-sort or attention fill.

**What to build:** The ambient clash mood from ADR 0002.

- [ ] Backend: two different sessions edit the same absolute path within 5 minutes (from AV-54's edit payloads); both get the startled mood with the file name; expires after a short hold
- [ ] Frontend: both bubbles show the startled emote and file name, linked per the AV-63 clash design
- [ ] No warning, no blocking, no trail entry
- [ ] `cargo test`: same file within window, outside window, same session twice (no clash), three sessions

AV-54 finding: absolute path is `tool_input.file_path` for `Edit`, `Write` and `MultiEdit`, but `tool_input.notebook_path` for `NotebookEdit`; `PostToolUse` also echoes `tool_response.filePath`. Count only `PostToolUse` (a `PostToolUseFailure` edit changed nothing). Fixtures: `post_tool_use_edit.json`, `post_tool_use_write.json`, `pre_tool_use_notebook_edit.json`.

**Done when:** two live sessions editing one file show the clash in `npm run tauri dev` and the owner confirms it.
