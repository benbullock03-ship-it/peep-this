---
id: AV-69
title: Backend mood engine
status: done
depends: [AV-54, AV-56, AV-65]
attended: false
---

# AV-69 — Moods

**What to build:** Each session in the snapshot carries at most one mood, derived purely from inputs.

- [x] Tired (owner ruling 2026-11-15): sweat from 70% while below the nudge setting; yawn from the nudge setting (AV-70: 50 to 95, default 80), which is the context nudge; the second nudge once at 95%
- [x] Worry: a `Bash` whose command matches the repo setup's test command and failed. Per AV-54, a failed Bash fires `PostToolUseFailure`, not `PostToolUse`: `tool_input.command` plus `error` (`"Exit code N\n<stderr>"`) and `is_interrupt`; there is no numeric exit code field, so failure is the event itself (fixture `post_tool_use_failure_bash.json`). Successful `PostToolUse` has `tool_response.stdout/stderr/interrupted`, no exit code
- [x] Cheer: a ticket in that repo moved to closed (AV-65); the other sessions cheer briefly
- [x] Greet: a new session appeared; the sessions already open wave briefly
- [x] Nap: idle past a threshold (zzz); clears on the next prompt
- [x] Priority between moods fixed in one place; attention states always win (frontend hides the mood)
- [x] `cargo test` for each mood, expiry, and priority

Decision taken for the owner to overrule: Coderot's "no survivors" cheer is left out of v1, since the widget has no signal for it without the skill writing a file.

**Done when:** a fixture-driven test covers each mood and the snapshot carries them live.

## Done 2026-11-15

`src-tauri/src/mood.rs` (`derive`, `is_test_run`, one test per mood, expiry and priority); `Session.mood` set by `refresh_moods` in `store.rs` after every change, `set_repos` and tick; failed or passed test runs reach `state.rs` as `Event::TestRun` from `ingest/server.rs` (`test_run`, reading `tickets::setup_test`). Fixture-driven: `test_command_runs_wrap_as_test_run_and_worry_the_session_live` (`post_tool_use_failure_bash.json`, `post_tool_use_bash.json` through the store), `moods_refresh_live_on_changes_and_expire_on_tick` (greet, expiry, cheer). Spec §5.1, §5.3, §13.5 updated. `cargo test` 357 pass, `npm test` 200 pass, `npm run build` ok.

## Decisions taken without a ruling

- Mood values: `cheer`, `greet`, `worry`, `yawn`, `sweat`, `nap`. Tired is two values (sweat, yawn); the second nudge at 95% is left to AV-70 to read from `contextPct`.
- Nudge fixed at 80 (`mood::NUDGE_PCT_DEFAULT`) until AV-70 adds the setting; a nudge under 70 yawns with no sweat band.
- Priority: cheer, greet, worry, yawn, sweat, nap. The two brief social moods go first so they are seen at all.
- Mood is still set during an attention state; the frontend hides it, per the ticket.
- Worry: a run matches when the Bash/PowerShell command contains any non-`cd` step of the setup's `test` line; holds 5 min, cleared by a passing matching run; an interrupted run is neither pass nor fail.
- Cheer: 8s, for sessions in the repo except the one whose linked ticket is the closed one.
- Greet: 5s, only by sessions born at least 2s before the newcomer, so sessions found in one startup scan don't greet each other.
- Nap: after 10 min in `idle` or `success`; ended sessions have no mood.
- The frontend `Session` type is left for AV-70.
