# backend

## How it works now

- Rust in `src-tauri/` owns all truth: session files, hook and statusline POSTs, transcripts, all run through the pure state function in `state.rs`; the frontend gets a full `sessions` snapshot.
- Each session carries at most one mood (`mood.rs` `derive`): `cheer`, `greet`, `worry`, `yawn`, `sweat`, `nap`, in that priority. `store.rs` `refresh_moods` recomputes it after every change, `set_repos` and tick.
- Test runs arrive as `Event::TestRun` from `ingest/server.rs`: a Bash/PowerShell command matching a non-`cd` step of the repo setup's `test` line (`tickets::setup_test`).
- Worry holds 5 min or until a passing matching run; cheer 8s for other sessions in a repo when a ticket closes; greet 5s; nap after 10 min in `idle` or `success`.

## Watch out

- Mood never changes state, and is still set during attention states; the frontend hides it there.
- Greet only comes from sessions born 2s or more before the newcomer, so one startup scan doesn't greet itself.
- Nudge is fixed at 80% (`mood::NUDGE_PCT_DEFAULT`) until a setting exists; ended sessions have no mood.

## Why it's like this

- Cheer and greet outrank the rest because they are brief and would otherwise never be seen.

last touched: AV-69, 2026-11-15
