# backend

## How it works now

- Rust in `src-tauri/` owns all truth: session files, hook and statusline POSTs, transcripts, all run through the pure state function in `state.rs`; the frontend gets a full `sessions` snapshot.
- Hooks are `type: "command"` entries running the relay `peepthis-statusline.exe hook <Event>`: it POSTs stdin to `/hook/<Event>` with the port and token from `endpoint.json`, prints the response (the `PermissionRequest` decision) and always exits 0.
- Each session carries at most one mood (`mood.rs` `derive`): `cheer`, `greet`, `worry`, `yawn`, `sweat`, `nap`, in that priority. `store.rs` `refresh_moods` recomputes it after every change, `set_repos` and tick.
- Test runs arrive as `Event::TestRun` from `ingest/server.rs`: a Bash/PowerShell command matching a non-`cd` step of the repo setup's `test` line (`tickets::setup_test`).
- Worry holds 5 min or until a passing matching run; cheer 8s for other sessions in a repo when a ticket closes; greet 5s; nap after 10 min in `idle` or `success`.
- Ended sessions go to the closed shelf (`closed.rs` `Shelf`): `store.rs` hands the session over as it was on end, `closed::entry` records its end-state flags (uncommitted, no Pack Up, attention, ticket doing), and `Snapshot.closed` carries the shelf to the frontend (`ClosedSession` in `src/model.ts`). Wired in `lib.rs` `wire_closed`.
- `end_session(session_id)` (`focus.rs`, a Tauri command only, never on the event server) re-reads `sessions/<pid>.json`, compares `procStart` with the OS start time and kills through the same process handle. `Ok(true)` killed, `Ok(false)` already gone, `Err` otherwise; the normal lifecycle then takes the row to done and fades it.

## Watch out

- The token never goes in `settings.json`; install and uninstall recognise both old http hook entries and command ones, so upgrades replace them.
- The relay's `PermissionRequest` read waits 70s (hold 60s, under the 75s hook timeout); the constant is duplicated because the bin cannot include `approve.rs`.
- Hook commands are shell form and rely on Git Bash, like the statusLine.

- Mood never changes state, and is still set during attention states; the frontend hides it there.
- Greet only comes from sessions born 2s or more before the newcomer, so one startup scan doesn't greet itself.
- Nudge is fixed at 80% (`mood::NUDGE_PCT_DEFAULT`) until a setting exists; ended sessions have no mood.
- The shelf persists across restarts and prunes on both load and write.
- `end_session` fails closed: no session file, no `procStart` or an unreadable start time refuses. A session with no session file (a child `claude` that inherited `CLAUDE_CODE_CHILD_SESSION`) exists only through hooks, so it cannot be ended and its row stays live after its process dies.

## Why it's like this

- Cheer and greet outrank the rest because they are brief and would otherwise never be seen.
- Command hooks instead of http hooks: Claude Code reported every refused connection as a hook error while the widget was closed; the relay stays silent.

last touched: AV-96, 2026-11-16
