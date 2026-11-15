---
id: AV-49
title: Shells from a subagent-run session never end
status: in-progress
depends: []
attended: false
---

# AV-49 — Shells from a subagent-run session never end

**What to build:** Background shells started by a session that is itself run by an agent stay `RUNNING` until the widget stops. Found in AV-30's live check (see its "Owed before done"); AV-29's own check used an interactive top-level session, which is not affected.

Two causes, both in the AV-29 tracking:

- The shell's `<task-notification>` goes to the agent, not through a `UserPromptSubmit` hook, so the widget never sees the end.
- The output-file fallback looks under the hook's `session_id`, but the files are under the parent's folder (`…\claude\<hash>\<parent session id>\tasks\`), so it finds nothing.

- [ ] Fixture: hook payloads and a task output file from a subagent-run session, anonymised
- [x] Locate the output file for such a session (parent session id, or search the hash folder for the task id)
- [x] A shell whose output file has its trailer ends as `exited` or `failed` with the exit code, with no hook
- [x] `cargo test` for both layouts; the interactive case keeps passing

**Done when:** a failing and a clean background command started from an agent-run session both ended in the widget on their own.

## Built 2026-11-15

`output_file` in `src-tauri/src/source/claude/shells.rs` looks under the hook's session id first, then under any session folder for the task id. `state.rs` already ended a shell from a probe's exit trailer without a hook, so finding the file was the whole fix. Test: `output_file_under_another_sessions_folder_still_ends_the_shell` (failing, clean, own-folder-wins, missing).

Also seen 2026-11-15: a top-level session after `/clear` keeps writing under its old id's folder, so the same miss hit interactive sessions too. The search covers it.

## Decisions taken without a ruling

- Search by task id rather than resolve the parent id: nothing in the hook payload names the parent, and task ids are random.
- No fixture files: the test builds the folder layout inline, as the existing output-file test does. No hook payload from an agent-run session was captured.

## Owed before done

- Live: a failing and a clean background command from an agent-run session, both ending in the reinstalled widget on their own.
