---
id: AV-49
title: Shells from a subagent-run session never end
status: todo
depends: []
attended: false
---

# AV-49 — Shells from a subagent-run session never end

**What to build:** Background shells started by a session that is itself run by an agent stay `RUNNING` until the widget stops. Found in AV-30's live check (see its "Owed before done"); AV-29's own check used an interactive top-level session, which is not affected.

Two causes, both in the AV-29 tracking:

- The shell's `<task-notification>` goes to the agent, not through a `UserPromptSubmit` hook, so the widget never sees the end.
- The output-file fallback looks under the hook's `session_id`, but the files are under the parent's folder (`…\claude\<hash>\<parent session id>\tasks\`), so it finds nothing.

- [ ] Fixture: hook payloads and a task output file from a subagent-run session, anonymised
- [ ] Locate the output file for such a session (parent session id, or search the hash folder for the task id)
- [ ] A shell whose output file has its trailer ends as `exited` or `failed` with the exit code, with no hook
- [ ] `cargo test` for both layouts; the interactive case keeps passing

**Done when:** a failing and a clean background command started from an agent-run session both ended in the widget on their own.
