---
id: AV-27
title: Per-subagent state tracking
status: todo
depends: []
attended: false
---

# AV-27 — Per-subagent state tracking

**What to build:** Stub. Replace the subagent count with a tracked list (spec §11.6). Each subagent gets its own state, derived by the same pure function from events carrying its `agent_id`.

- [ ] **Verify** `SubagentStart` accepts `http` hooks; if not, create the child on `PreToolUse Agent` and bind `agent_id` on its first tool event
- [ ] **Verify** what `SubagentStop` carries that can serve as a result line (last message? transcript path?)
- [ ] `Session.agents: Agent[]` with id, type, description, state, task, startedAt, endedAt, tool count, result line; `subagents` count kept until AV-28 lands
- [ ] Subagent tool events stop overwriting the parent's task
- [ ] Nested subagents and background agents that outlive the parent turn: record what the events look like, then decide

**Done when:** logs for a live session that fanned out three agents show three children with their own states and tools, each ending with a result line.
