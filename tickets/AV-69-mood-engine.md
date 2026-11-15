---
id: AV-69
title: Backend mood engine
status: open
depends: [AV-54, AV-56, AV-65]
attended: false
---

# AV-69 — Moods

**What to build:** Each session in the snapshot carries at most one mood, derived purely from inputs.

- [ ] Tired: context used from 70% (sweat), from 90% (yawn); this is the context nudge
- [ ] Worry: a `Bash` whose command matches the repo setup's test command and failed. Per AV-54, a failed Bash fires `PostToolUseFailure`, not `PostToolUse`: `tool_input.command` plus `error` (`"Exit code N\n<stderr>"`) and `is_interrupt`; there is no numeric exit code field, so failure is the event itself (fixture `post_tool_use_failure_bash.json`). Successful `PostToolUse` has `tool_response.stdout/stderr/interrupted`, no exit code
- [ ] Cheer: a ticket in that repo moved to closed (AV-65); the other sessions cheer briefly
- [ ] Greet: a new session appeared; the sessions already open wave briefly
- [ ] Nap: idle past a threshold (zzz); clears on the next prompt
- [ ] Priority between moods fixed in one place; attention states always win (frontend hides the mood)
- [ ] `cargo test` for each mood, expiry, and priority

Decision taken for the owner to overrule: Coderot's "no survivors" cheer is left out of v1, since the widget has no signal for it without the skill writing a file.

**Done when:** a fixture-driven test covers each mood and the snapshot carries them live.
