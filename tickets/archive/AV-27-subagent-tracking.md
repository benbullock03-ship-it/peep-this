---
id: AV-27
title: Per-subagent state tracking
status: done
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

## Verified 2026-11-14 (Claude Code 2.1.325)

Sources: the hooks docs (`code.claude.com/docs/en/hooks.md`: hook-type support table, SubagentStart, SubagentStop, Agent tool `tool_response`), plus a capture server on 127.0.0.1:47927 logging every event, driven by `claude -p ... --settings <scratch> --setting-sources project --allowedTools "Agent Read" --model haiku`. Run 1 launched three parallel agents (each read one file). Run 2 launched one `run_in_background` agent and asked it to nest another agent.

- **`SubagentStart` accepts `http`.** The docs list it under "command, http and mcp_tool", and every launch POSTed it live. Payload: common fields + `agent_id`, `agent_type`; **no `description`, no `tool_use_id`**. It arrives about 10ms after the parent's `PreToolUse Agent`, and launches pair up strictly: Pre(a), Start(a), Pre(b), Start(b), and so on. The fallback in the Verify line is therefore not needed, but it is built anyway for installs without the hook (below).
- **Result line:** `SubagentStop.last_assistant_message` holds the subagent's final text ("banana", or "The first line of b.txt is `banana`.\n\nI did not…"). It also carries `agent_transcript_path` (`…/<session>/subagents/agent-<id>.jsonl`) and `background_tasks`. In auto mode (v2.1.271+), a `SubagentHandback` subagent puts its real report in the `PreToolUse SubagentHandback` `tool_input.message`, and `last_assistant_message` is only its closing text (docs; not seen live).
- Subagent tool events come under the parent `session_id` with `agent_id` + `agent_type` (as in AV-06). Parallel agents interleave freely: b's Read ran first, then c's, then a's.
- The parent's `PostToolUse Agent` carries `tool_response.agentId` (also `status: "completed"`, `totalToolUseCount`, `totalDurationMs`, `content`). It is not used.
- No parent `UserPromptSubmit` fired on a foreground hand-back (AV-06 saw one). A **background** agent's completion does fire one (`<task-notification>` prompt), followed by its own turn and `Stop`.
- **Background agents:** `PreToolUse Agent` (`run_in_background: true`), then `SubagentStart`, then an immediate parent `PostToolUse Agent` with `status: "async_launched"`, `agentId`, `outputFile`. Then the parent's **`Stop`, while the agent is still running** (`background_tasks: [{id, type:"subagent", status:"running", description, agent_type}]`). The agent's tool events and `SubagentStop` follow after that `Stop`.
- **Nested agents:** not observed. The background agent declined to delegate. Per the docs, `agent_id` is "the subagent … the hook fires in", so a nested agent's events would carry its own `agent_id` under the same `session_id`, and its launching `PreToolUse Agent` would carry the outer agent's `agent_id`.
- An internal `SubagentStop` (prompt suggestions, `/btw`) has an empty `agent_type` and is not preceded by `SubagentStart` (docs + AV-06).
- Fixtures: `subagent_fanout.jsonl` (run 1, 21 events) and `subagent_background.jsonl` (run 2, 11 events), as `{"hook", "body"}` lines; `subagent_pre_tool_use_read.json`, `subagent_post_tool_use_read.json`, `post_tool_use_agent_async.json`. `subagent_stop.json` now has a multi-line `last_assistant_message`.

## Built

- `Event::SubagentStart { agent_id, agent_type }`, `Event::SubagentStop { agent_id?, result? }`, and `Event::Subagent { agent_id, agent_type, event }`, which wraps `PreToolUse`/`PostToolUse`/`Notification` (incl. `PermissionRequest`) whenever the payload has `agent_id` (`ingest/server.rs::scope_to_agent`). `SubagentStart` is added to the installed hook events (11 now). Existing installs therefore read `outdated` until reinstalled.
- `Session.agents: Agent[]`: `id`, `agentType`, `description`, `state`, `stateSince`, `task`, `lastTool?`, `startedAt`, `endedAt?`, `toolCount`, `result?`, derived in `state::apply`. An agent starts `thinking`, moves to `working` on its PreToolUse and back to `thinking` on its PostToolUse, goes `waiting` on its permission, goes `stuck` after 6 min quiet (ticker now also ticks sessions with busy agents), and ends `success` "Done" with `result` on `SubagentStop`. Session `Ended` ends running agents as `idle`. `subagents` count unchanged until AV-28.
- Subagent tool events no longer touch the parent's task, last tool or state. They still reset the parent's stuck timer and hook freshness.
- Proof: `ingest/server.rs` tests `replayed_three_agent_fanout_gives_three_children_with_states_tools_and_results`, `replayed_fanout_midway_shows_each_child_in_its_own_state` and `replayed_background_agent_outlives_the_turn` replay the captured sequences through `handle()` into the store. `cargo test replayed -- --nocapture` prints the per-event log: three children, each with its own Working/Thinking, `Read <file>`, tools=1, and result `apple`/`banana`/`cherry`. Plus 14 `state.rs` agent tests and the extended fixture mapping test.

## Decisions taken without a ruling

- Description binding: the parent's (or an agent's) `PreToolUse Agent` queues its `description`, and the next new `agent_id` (from `SubagentStart`, or its first event when that hook is not installed) takes the oldest one. With `SubagentStart` installed this follows the verified strict pairing. Without it, the binding is best-effort, because parallel agents' first tools come in any order. The queue holds 8 and is cleared on `Stop`.
- Result: the first non-blank line of `last_assistant_message`, clipped to 200 chars. A `SubagentHandback` `tool_input.message` (first line) wins when present. This is a new derived string kept beyond `task` (spec §6.5 change below).
- Background agents: `Stop` leaves `agents` alone. A background agent keeps its own state after the parent shows `success`, and the parent row's state is not changed by its tool events.
- Nested agents: flat. Every `agent_id` is a sibling under the session; no parent-agent link is kept.
- A subagent's permission request also puts the **parent** in `waiting`, so the toast and alert still fire per session. The parent returns to its prior state (for example `working` "Agent …", or `success` for a background agent) on that agent's next tool event or `SubagentStop`. Another agent's events do not release it.
- `SubagentStop` without `agent_id`, or for an unknown id, creates no agent (internal agents). Events for an already finished agent are ignored. A `SubagentStart` for a finished id (resume, agent-team teammate) restarts it and keeps its description.
- Finished agents stay in the list (UI linger and the "done N" counter are AV-28's job). The list keeps at most 20 finished per session, dropping the oldest first.
- `src/model.ts` was not extended: the build type-checks without it, and AV-28 adds the type when it renders.
- Parent still shows `thinking` after its first `PostToolUse Agent` while sibling agents run (existing rule). AV-28's child rows make that visible.

## Live check (2026-11-14)

Run from an interactive Claude Code session at the owner's request, against `npm run tauri dev`. The integration already had `SubagentStart`. Three parallel Explore agents logged three `SubagentStart` and three `SubagentStop` hooks. The snapshot went from `[Explore=Thinking tools=0 …]` x3 through each agent's own Working/Thinking to `[Explore=Success tools=2 result=Some("22"); Explore=Success tools=2 result=Some("23"); Explore=Success tools=2 result=Some("20")]`. The parent showed `Thinking`/`Working` rather than `Working "Agent …"`, as decision 52 expects.

Original instruction: reinstall the integration (status shows `outdated` until then, since `SubagentStart` is new), run `npm run tauri dev`, and in an interactive session ask for three parallel subagents that each read a file. The widget log's snapshot lines should show the parent `Working "Agent …"` while three agent entries go through their own states and finish with result lines. The snapshot log line now appends `[type=State tools=N result=…; …]` per session with agents (orchestrator edit to `lib.rs::log_snapshot`), so the check reads straight from the log.
