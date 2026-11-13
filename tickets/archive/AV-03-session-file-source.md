---
id: AV-03
title: Claude session-file source, store and snapshot event
status: done
depends: [AV-02]
attended: false
---

# AV-03 — Claude session-file source, store and snapshot event

**What to build:** The `AgentSource` trait and its Claude implementation that watches `~/.claude/sessions/*.json` (spec §6.1), a store keyed by `sessionId`, and a throttled `sessions` Tauri event carrying the full snapshot (spec §5.3).

- [x] `AgentSource` trait in `source/`; nothing outside `source/claude/` parses Claude formats
- [x] Lenient parser with fixtures copied from real session files (anonymise cwd/name)
- [x] Watch with `notify`, plus 5s poll safety net
- [x] Liveness by PID + `procStart`; dead process marks the session ended (`alive: false`, `endedAt`)
- [x] Store emits snapshot on change, at most every 100ms
- [x] `get_snapshot` command so the frontend can fetch on load

**Done when:** `cargo test` covers parsing and liveness, and `npm run tauri dev` logs a snapshot listing the live sessions on this machine.

## Decisions taken without a ruling

- Session files also carry `status: "shell"`; only `busy`/`idle` map to `SessionFile { busy }`, any other status sends no state event (session keeps its current state).
- `procStart` is the process creation FILETIME (100ns since 1601) as a string; liveness compares it to `GetProcessTimes` with 2s tolerance. Missing `procStart` falls back to PID-only.
- Liveness uses `windows-sys` (`OpenProcess` + `GetExitCodeProcess` + `GetProcessTimes`), not `sysinfo`. Access denied counts as alive. Non-Windows always reports alive (untested platform).
- Generic process probe lives in `source/process.rs` so future adapters can reuse it; Claude's `procStart` interpretation stays in `source/claude/sessions.rs`.
- A session file whose PID is already dead when first seen is ignored (stale files from old runs never appear); only a session seen live and then dead/removed emits `Ended`.
- A file that fails to parse while its session is known keeps the session alive (caught mid-write); malformed files are logged once per path, not every poll.
- Watcher does a full rescan on any `.json` change (50ms debounce) and every 5s; the 5s rescan re-sends `busy` so the session-file fallback recovers once hooks go stale. Store suppresses emits when nothing changed.
- `CLAUDE_CONFIG_DIR` is honoured in place of `~/.claude`.
- An `Upsert` for a session id that has ended starts a fresh session (covers resume reusing the id); `Ended` and `apply` for unknown ids are dropped.
- Missing `name` falls back to the cwd basename, then `claude`; missing `pid` falls back to the file stem.
- Snapshot sessions are ordered by id; display sort stays in the frontend. `integration` is `missing` and `rateLimits` absent until AV-07/AV-08.
- Ended sessions are not pruned here; removal 60s after `endedAt` is AV-10.
