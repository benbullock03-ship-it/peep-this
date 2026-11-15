---
id: AV-64
title: Backend links each session to its running skill and ticket
status: done
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

## Decisions taken without a ruling

- Snapshot fields: `skill?: string` (no `peep:` namespace) and `ticket?: { id, title? }`; state input is `Event::Skill { skill, ticket, event }` wrapping the hook event, like `Event::Shell`.
- Only the six Peep skills of spec §13.1 link (`PEEP_SKILLS` in `state.rs`); any other `peep:` name and other plugins' skills are ignored.
- A ticket id is the first `PREFIX-N` argument, uppercased (`close av-12` links `AV-12`); a bare number (`/peep:ticket 34`) links nothing until the repo setup carries a prefix.
- A `/peep:` prompt replaces the ticket link (none when it has no id); a mid-turn `Skill` tool call sets the skill and only replaces the ticket when its args carry an id.
- Any `UserPromptSubmit` clears the skill before a `/peep:` prompt sets it, so an interrupted run (no `Stop`) does not leave the floppy in; the session ending clears both.
- Tickets folder from the `tickets:` line of `.peep/setup.md` (AV-56 format), else `tickets/`; title is frontmatter `title:` of `<id>-*.md` in that folder (archive not searched). Ticket reading lives in `src-tauri/src/tickets.rs`, hook parsing in `source/claude/skill.rs`.
