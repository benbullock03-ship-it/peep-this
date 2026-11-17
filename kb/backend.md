# backend

## How it works now

- Rust in `src-tauri/` owns all truth: session files, hook and statusline POSTs, transcripts, all run through the pure state function in `state.rs`; the frontend gets a full `sessions` snapshot.
- Hooks are `type: "command"` entries running the relay `peepthis-statusline.exe hook <Event>`: it POSTs stdin to `/hook/<Event>` with the port and token from `endpoint.json`, prints the response (the `PermissionRequest` decision) and always exits 0.
- Each session carries at most one mood (`mood.rs` `derive`): `startled` (clash), `cheer`, `greet`, `tweet`, `worry`, `yawn`, `sweat`, `nap`, in that priority. `store.rs` `refresh_moods` recomputes it after every change, `set_repos` and tick, then `mood::gate` drops all but `startled` while the `moods` setting is off (the default; `moods_on`/`set_moods_on`, key `moods` in the widget `settings.json`).
- Test runs arrive as `Event::TestRun` from `ingest/server.rs`: a Bash/PowerShell command matching a non-`cd` step of the repo setup's `test` line (`tickets::setup_test`).
- Worry holds 5 min or until a passing matching run; cheer 8s for other sessions in a repo when a ticket closes; greet 5s; nap after 10 min in `idle` or `success`.
- Ended sessions go to the closed shelf (`closed.rs` `Shelf`): `store.rs` hands the session over as it was on end, `closed::entry` records its end-state flags (uncommitted, no Pack Up, attention, ticket doing), and `Snapshot.closed` carries the shelf to the frontend (`ClosedSession` in `src/model.ts`). Wired in `lib.rs` `wire_closed`.
- A session file that fails to parse on two scans in a row is kept from its live PID with `SessionInfo.unparsed` (`sessions.rs` `scan_with`): a known session keeps its last info and state; one first seen broken is salvaged (`salvage`: `sessionId`/`cwd` from the broken text, else `unparsed-<pid>`) and starts `idle`. The next good parse clears the mark; the row shows a `?` chip.
- `end_session(session_id)` (`focus.rs`, a Tauri command only, never on the event server) re-reads `sessions/<pid>.json`, compares `procStart` with the OS start time and kills through the same process handle. `Ok(true)` killed, `Ok(false)` already gone, `Err` otherwise; the normal lifecycle then takes the row to done and fades it.

- Signpost (AV-129): `transcript.rs` emits `Event::TranscriptNext` for a main-thread assistant text block whose last line is `next: <value>`; `state.rs` keeps it in `Session.next` only once a Peep skill has run in that session (`peep_ran`), clears it when a new Peep skill starts, and keeps it across plain turns. `Repo.setup` says whether `.peep/setup.md` exists.

- Claude Code fixtures live per version in `tests/fixtures/<version>/{hooks,sessions,transcripts}`; the hand-made ones are `legacy/`, which the unit tests read. `settings/` and `tickets/` stay flat. `source/claude/mod.rs` `contract::every_version_folder_parses` runs the parsers over every version folder (`malformed*` skipped; hooks routed by `hook_event_name`, `statusline*` by name).
- Version gate: `build.rs` bakes the `N.N.N` fixture folder names into `PEEP_FIXTURE_VERSIONS`; the source sink in `lib.rs` hands each `Upsert`'s `SessionInfo.version` to `tray::version_notice`, which adds one disabled "untested Claude Code version <major.minor>" line atop the tray menu for the first major.minor no folder covers. Rows, state and snapshot never see it.
- `npm run capture` (`scripts/capture.mjs`) checks a new Claude Code release: it copies live session files, points `endpoint.json` at itself for 60s to record up to 3 POSTs per route while forwarding them to the widget, and writes them anonymised (`scripts/anonymise.mjs`) to the session's version folder.

## Watch out

- A new `npm run capture` folder only clears the gate after a rebuild (`build.rs` reruns when `tests/fixtures` changes); the tray line stays until restart and names only the first untested version.
- Capture redacts free text (`prompt`, `content`, `tool_response`, ...) to `<redacted>`, so captured fixtures prove shape, not semantics; a widget restart mid-capture keeps its own `endpoint.json` and capture leaves it alone.

- A first transcript scan ignores `next:` lines, and `peep_ran` is not persisted, so a widget restart shows no signpost until the next Peep skill run.

- The token never goes in `settings.json`; install and uninstall recognise both old http hook entries and command ones, so upgrades replace them.
- The relay's `PermissionRequest` read waits 70s (hold 60s, under the 75s hook timeout); the constant is duplicated because the bin cannot include `approve.rs`.
- Hook commands are shell form and rely on Git Bash, like the statusLine.

- Mood never changes state, and is still set during attention states; the frontend hides it there.
- Greet only comes from sessions born 2s or more before the newcomer, so one startup scan doesn't greet itself.
- The context nudge is the yawn's bubble, so with moods off it is gone too; ended sessions have no mood.
- The gate filters only the winning mood; clash outranks every other mood, so it is never hidden.
- The shelf persists across restarts and prunes on both load and write. An unparsed session never reaches it (its id may be made up).
- An unparsed session is kept on PID liveness alone, without the `procStart` check, so PID reuse is not guarded while its file is broken. Malformed hook bodies and transcript lines never drop a session; they are skipped.
- `end_session` fails closed: no session file, no `procStart` or an unreadable start time refuses. A session with no session file (a child `claude` that inherited `CLAUDE_CODE_CHILD_SESSION`) exists only through hooks, so it cannot be ended and its row stays live after its process dies.

## Why it's like this

- Cheer and greet outrank the rest because they are brief and would otherwise never be seen.
- Command hooks instead of http hooks: Claude Code reported every refused connection as a hook error while the widget was closed; the relay stays silent.

last touched: AV-136, 2026-11-17
