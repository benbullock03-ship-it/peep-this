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

**Done when:** the `sessions` snapshot shows skill and ticket for a live `/peep:ticket` run.
