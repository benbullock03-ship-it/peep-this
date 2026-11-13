---
id: AV-03
title: Claude session-file source, store and snapshot event
status: todo
depends: [AV-02]
attended: false
---

# AV-03 — Claude session-file source, store and snapshot event

**What to build:** The `AgentSource` trait and its Claude implementation that watches `~/.claude/sessions/*.json` (spec §6.1), a store keyed by `sessionId`, and a throttled `sessions` Tauri event carrying the full snapshot (spec §5.3).

- [ ] `AgentSource` trait in `source/`; nothing outside `source/claude/` parses Claude formats
- [ ] Lenient parser with fixtures copied from real session files (anonymise cwd/name)
- [ ] Watch with `notify`, plus 5s poll safety net
- [ ] Liveness by PID + `procStart`; dead process marks the session ended (`alive: false`, `endedAt`)
- [ ] Store emits snapshot on change, at most every 100ms
- [ ] `get_snapshot` command so the frontend can fetch on load

**Done when:** `cargo test` covers parsing and liveness, and `npm run tauri dev` logs a snapshot listing the live sessions on this machine.
