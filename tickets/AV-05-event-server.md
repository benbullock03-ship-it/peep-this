---
id: AV-05
title: Localhost event server with token
status: todo
depends: [AV-02]
attended: false
---

# AV-05 — Localhost event server with token

**What to build:** `ingest/server.rs` per spec §6.5: `127.0.0.1` listener on 47821 (random fallback), `endpoint.json` with port and token in `%APPDATA%\AgentView\`, `POST /hook/<Event>` and `POST /statusline` routed into the store as `Event`s.

- [ ] Binds loopback only
- [ ] Token generated per app start, compared in constant time; 401 otherwise
- [ ] 1MB body cap; payload reduced to the fields the state engine needs, the rest dropped
- [ ] Events for unknown `session_id` create a placeholder session that the session-file source fills in
- [ ] Tests: token accept/reject, body cap, routing

**Done when:** `cargo test ingest` passes and a manual `curl` with the token moves a live session's state in the logs.
