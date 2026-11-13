---
id: AV-05
title: Localhost event server with token
status: done
depends: [AV-02]
attended: false
---

# AV-05 — Localhost event server with token

**What to build:** `ingest/server.rs` per spec §6.5: `127.0.0.1` listener on 47821 (random fallback), `endpoint.json` with port and token in `%APPDATA%\AgentView\`, `POST /hook/<Event>` and `POST /statusline` routed into the store as `Event`s.

- [x] Binds loopback only
- [x] Token generated per app start, compared in constant time; 401 otherwise
- [x] 1MB body cap; payload reduced to the fields the state engine needs, the rest dropped
- [x] Events for unknown `session_id` create a placeholder session that the session-file source fills in
- [x] Tests: token accept/reject, body cap, routing

**Done when:** `cargo test ingest` passes and a manual `curl` with the token moves a live session's state in the logs.

## Decisions taken without a ruling

- Check order is token (401) first, then path (404), method (405), size (413), JSON (400). Unauthenticated callers learn nothing about routes. Success is 204.
- Unknown hook names (e.g. `PreCompact`) return 204 and are dropped. `PostToolUseFailure` maps to `PostToolUse`.
- Notification `idle` = `notification_type == "idle_prompt"`; if the field is absent, the message contains "waiting for your input".
- `StopFailure` summary is the first string of `error`, `error.message`, `error_message`, `message`, `reason`. The field is unverified; adjust when AV-06 sees a real payload.
- PreToolUse target is the first non-empty of `file_path, notebook_path, command, pattern, path, url, query, description, subagent_type`. Paths under the session cwd become relative. Every kept string is cut to its first line and 200 chars. `prompt`, tool outputs and other fields are never read.
- Placeholder sessions (unknown `session_id`) have pid 0, name = cwd basename (or the first 8 chars of the id), empty kind/entrypoint, and state from the event. A session-file upsert fills identity and keeps the hook state. `SessionEnd` for an unknown id creates nothing. `SessionStart` on an ended session revives it with the same identity.
- `Store::apply` still drops unknown ids (sources and transcripts use it). Hooks and statusline use the new `Store::apply_or_placeholder`.
- Placeholders that never get a session file never expire. AV-10 (timers) should end pid-0 sessions after a timeout.
- Statusline rate limits are stored only when both `five_hour` and `seven_day` are present. A change in `at` alone does not trigger an emit.
- Context % uses `100 - remaining_percentage`, or `used_percentage` if remaining is absent.
- Every routed request is logged as `agentview: <hook Name|statusline> session <id>`, which includes statusline ticks. If that gets noisy, quieten statusline logging in AV-07.
- Verified 2026-11-13 against the running app: no token returned 401; with the token, `POST /hook/Notification` returned 204, the log showed `agentview: hook Notification session <id>`, and a snapshot was emitted for the live session.
- `endpoint.json` is written as a temp file and then renamed. The default dir is `%APPDATA%\AgentView`, or the temp dir if `APPDATA` is unset.
