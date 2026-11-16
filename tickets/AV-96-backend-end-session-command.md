---
id: AV-96
title: Backend end_session command
status: done
depends: [AV-95]
---

# AV-96 — Backend end_session command

**What to build:** A Tauri command that ends one session's process. It is called only from the widget UI and is never reachable over HTTP.

- [x] `end_session(session_id)` beside `focus_session` (`src-tauri/src/focus.rs`): resolve the PID from the store, TerminateProcess it, return whether it worked
- [x] Refuse if the PID's process start time no longer matches the session's `procStart`, so a reused PID is never killed
- [x] After the kill, the existing lifecycle (AV-10) takes the row to done and fades it; no new state
- [x] Not registered on the event server; `cargo test` for the PID-mismatch refusal and the unknown-session error

**Done when:** calling `end_session` on a throwaway `claude` process ends it, the row goes to done and fades, and a mismatched `procStart` returns an error without killing anything.

## Notes

- Tests in `focus.rs`: `end_allows_matching_start_time`, `end_refuses_reused_pid`, `end_fails_closed_when_start_cannot_be_compared`, `end_errors_for_unknown_session`, and on Windows `end_refusal_leaves_the_process_running` and `end_kills_a_verified_throwaway_process` (a spawned `ping`, never a claude session).
- The lifecycle item needs no code: after the kill the session watcher's next scan sees the PID dead and emits `Ended`, as for any exit.

## Done 2026-11-16

- Live check in `npm run tauri dev`, invoked over WebView2 CDP: an unknown id returned `no live session with that id`; a throwaway `claude` session (PID 34612) returned `true`, the PID was gone, the snapshot showed it not live (`7 sessions (6 live)`) and it was dropped within 75s.
- `procStart` mismatch is proven by `end_refuses_reused_pid` and `end_refusal_leaves_the_process_running`.
- Seen while testing: a session started with the parent's `CLAUDE_CODE_CHILD_SESSION` writes no `sessions/<pid>.json`; its hook-only row stays "live" after its process dies.

## Decisions taken without a ruling

- The store does not keep `procStart`, so `end_session` re-reads `<config dir>/sessions/<pid>.json` with the existing `sessions::parse` and compares through `sessions::is_live` (same 2s tolerance). To reach the config dir, `source::claude::config_dir` went from private to `pub`; store.rs and state.rs are untouched.
- Fails closed: no session file, no `procStart`, a session id that differs from the file's, or an unreadable OS start time all refuse with an error.
- The check and the kill use one process handle (`PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION`), so the PID cannot be reused between them.
- Return value: `Ok(true)` killed, `Ok(false)` the process had already exited (nothing to do, the lifecycle finishes the row), `Err(message)` for unknown session, unverifiable start, mismatch or a failed `TerminateProcess`. Exit code passed to `TerminateProcess` is 1.
