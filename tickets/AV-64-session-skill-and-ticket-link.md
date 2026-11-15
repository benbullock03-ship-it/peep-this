---
id: AV-64
title: Backend links each session to its running skill and ticket
status: open
depends: [AV-54]
attended: false
---

# AV-64 — Skill and ticket link per session

**What to build:** The snapshot carries which Peep skill a session is running and which ticket it's on.

- [ ] From the `UserPromptSubmit` prompt (per AV-54): parse `/peep:<skill> [args]`; a ticket id argument links the ticket
- [ ] Ticket title read from the session's repo tickets folder (repo setup, else `tickets/`); a missing file links the id alone
- [ ] Skill clears on `Stop` (the floppy ejects); the ticket link stays until the next `/peep:` prompt or the session ends
- [ ] Lives in `state.rs` as pure inputs and outputs; Claude-specific parsing stays in `source/claude/`
- [ ] `cargo test` with the AV-54 fixtures: link, eject on Stop, ticket kept across turns, unknown skill ignored

AV-54 finding: `prompt` is the slash command verbatim (`/caveman:caveman-help extra-arg-42`), not the expanded body. A skill Claude invokes itself arrives as `PreToolUse`/`PostToolUse` with `tool_name: "Skill"`, `tool_input.skill` (`"<plugin>:<skill>"`) and `tool_input.args`; treat it as a second source of the same link. Fixtures: `user_prompt_submit_slash_command.json`, `pre_tool_use_skill.json`, `post_tool_use_skill.json`.

**Done when:** the `sessions` snapshot shows skill and ticket for a live `/peep:ticket` run.
