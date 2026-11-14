---
id: AV-29
title: Background shell tracking with health states
status: done
depends: []
attended: false
---

# AV-29 — Background shell tracking with health states

**What to build:** Stub. Track each background shell a session starts and derive one of five states (spec §11.7): `running`, `quiet`, `serving`, `exited`, `failed`.

- [ ] **Verify** how a background shell appears in hook payloads: the `run_in_background` input, the shell id in the tool response, the tools that read output and kill it, and whether any event reports exit and exit code
- [ ] **Verify** whether the shell's OS PID can be found (child of the session PID with a matching command line)
- [ ] `quiet`: no output observed for 2 min (only if output is observable; else drop the state and record why)
- [ ] `serving`: a process in the shell's tree holds a listening TCP port; show the port
- [ ] `failed`: non-zero exit, with exit code and last error line if available
- [ ] `Session.shells: Shell[]` with id, command, state, startedAt, port, exit code, last line
- [ ] Shells end with the session; orphan tracking is out of scope

**Done when:** logs for a live session running a dev server and a failing build show `serving :<port>` for one and `failed` with its exit code for the other.

## Verified 2026-11-14 (Claude Code 2.1.325)

Sources: hooks docs (Bash/PowerShell `tool_input`, Stop `background_tasks`), tools reference (background commands, `TaskOutput` removed in 2.1.277, `TaskStop`), interactive-mode "How backgrounding works", env-vars (`CLAUDE_CODE_TMPDIR`). Plus a capture server on 127.0.0.1:47929 and two `claude -p ... --settings <scratch> --setting-sources project --allowedTools "Bash Read TaskStop" --model haiku` runs. Run 1: a `node -e` HTTP server on :47931 and `sleep 2; echo build broke >&2; exit 3` in the background, read both outputs, curl the server, `TaskStop` it. Run 2: `sleep 1; echo ok`, a server on :47932 that exits by itself after 45s, and `sleep 60` stopped with `TaskStop`. Reply before the server exits.

- **Start:** `PreToolUse Bash` has `tool_input.run_in_background: true`. The `PostToolUse` arrives at once with `tool_response.backgroundTaskId` (`"bx33ejiyl"`) and empty stdout/stderr. That id is the shell id everywhere else.
- **Output:** no hook carries it. Claude Code writes it to `<CLAUDE_CODE_TMPDIR or %TEMP%>\claude\<16-hex>\<session_id>\tasks\<task id>.output`. The docs say only "output is written to a file"; the path shows up in the notification and in Claude's `Read` of it. Claude reads output with `Read` on that file (`TaskOutput`/`BashOutput` are gone).
- **Exit and exit code:** a finished shell fires a `UserPromptSubmit` whose `prompt` is `<task-notification>` with `<task-id>`, `<tool-use-id>`, `<output-file>`, `<status>` (`completed` or `failed` seen) and `<summary>`: `Background command "<description>" failed with exit code 3` or `... completed (exit code 0)`. It fires mid-turn (right after the next tool) or as a new turn after `Stop` (run 2, followed by its own `Stop`). The output file ends with `[exited with code N]`.
- **Kill:** `PreToolUse/PostToolUse TaskStop` with `tool_input.task_id`; the response has `task_type: "local_bash"` and `command`. **No notification follows a stop.** The output file ends with `[killed]`.
- **`Stop.background_tasks`** lists running shells as `{id, type: "shell", status: "running", description, command}`; a shell still running at `Stop` is in it (run 2). Finished ones are not.
- **OS PID:** found. Each background shell is a direct child of the session's `claude.exe` (the session file PID): `bash.exe -c "source <snapshot> ... && eval '<command, re-escaped>' < /dev/null && pwd -P >| <cwd file>"`. The real process (here `node.exe`) sits 3 levels below it; the listening socket belongs to that descendant. The command shows up in the wrapper's command line, but with its quotes escaped differently.
- Not observed: a shell moved to the background by its timeout or `Ctrl+B` (docs say its result names the task id and output file; whether `backgroundTaskId` is set is unverified), the PowerShell tool's wrapper, a `/tasks` stop, and whether a stopped shell ever notifies.
- Fixtures: `shell_background.jsonl` (run 1, 20 events) and `shell_background_stop.jsonl` (run 2, 18 events), as `{"hook", "body"}` lines with prompts, outputs and replies replaced by `SECRET-*`; `post_tool_use_bash_background.json`, `user_prompt_submit_task_notification.json`, `post_tool_use_task_stop.json`, `stop_background_shell.json` (its subagent entry is copied from AV-27's capture).

## Built

- `Session.shells: Shell[]` with `id`, `command`, `description`, `state`, `stateSince`, `startedAt`, `endedAt?`, `port?`, `exitCode?`, `lastLine?`. States are `running`, `quiet`, `serving`, `exited`, `failed`, derived in `state::apply`.
- Hook side (`ingest/server.rs::shell_update`): a hook payload that also carries shell news becomes `Event::Shell { update, event }`. The shell update is applied first, then the wrapped event as usual. A `PostToolUse` of Bash/PowerShell with `backgroundTaskId` gives `Started`. A `<task-notification>` prompt gives `Ended` with `Code(n)` from the summary (`Failed`/`Killed`/`Unknown` without one) and still counts as `UserPromptSubmit`. `PostToolUse TaskStop` (or legacy `KillShell`) gives `Ended Killed`. Shell entries in `Stop.background_tasks` give `Listed`, which adopts unknown ids. Shells started by a subagent belong to the session too.
- OS side, kept outside the pure function: `ticker::probe_shells` runs before each 5s tick. It asks the store for `shell_queries()`, runs the probe outside the lock, and feeds one `Event::ShellProbe { id, obs }` per shell back through `state::apply`. Probe events never reset the stuck timer or hook freshness. The probe (`source/claude/shells.rs`, OS calls in `source/process.rs`):
  - finds the wrapper PID by matching letters and digits of the command against the command lines of the session PID's direct children (`NtQueryInformationProcess`), earliest-started unclaimed first, then keeps that PID;
  - finds the port: the lowest listening TCP port (IPv4 and IPv6, `GetExtendedTcpTable`) held by the wrapper or any descendant gives `serving`. Losing it drops back to `running`/`quiet`;
  - reads the output file: its mtime gives `quiet` after 2 min with no write. Its last non-blank line becomes `lastLine`, and its trailer (`[exited with code N]`/`[killed]`) ends the shell when no hook did, which covers `/tasks` stops. After a shell ends, it is probed once more so a failed shell gets its last error line.
- Ending a shell: `Code(0)`, `Killed` or `Unknown` give `exited`; a non-zero code or `Failed` gives `failed`. Session `Ended` ends every running shell as `exited`. At most 20 finished shells are kept, oldest dropped first.
- `log_snapshot` appends `{shells: <cmd>=State :port exit=N last="…"; …}` per session.
- Cargo: `windows-sys` features `Wdk_System_Threading` and `Win32_NetworkManagement_IpHelper`; no new crate.
- Proof: `cargo test replayed_dev_server -- --nocapture` replays run 1 through `handle()` into the store. After each request it runs the real probe logic (`probe_with`) against a fake process table (claude 100, then bash wrapper, then node holding :47931) and temp output files. At the curl it shows `b6s7b19v8=Serving port=Some(47931)` and `bx33ejiyl=Failed exit=Some(3) last=Some("build broke")`. `replayed_task_stop_completion_and_shell_outliving_the_turn` covers run 2. `real_probe_matches_a_child_shell_and_its_listening_port` runs the real Windows probe against a PowerShell child listening on a port. Also 10 state tests, parse tests, a ticker seam test and process tests.

## Decisions taken without a ruling

- `quiet` is kept, not dropped. Hooks never show output, but the output file's mtime does, so 2 min without a write gives `quiet`. The file path is undocumented (`<tmp>\claude\<hash>\<session>\tasks\<id>.output`, found by scanning `<tmp>\claude\*`). If Claude Code moves it, `quiet` and `lastLine` silently stop, and the trailer fallback goes with them; the other states still work.
- `serving` wins over `quiet`, since a dev server goes silent once it is up. The port shown is the lowest one in the tree (a node inspector port can show up next to the app port).
- The wrapper is matched by command-line substring after keeping letters and digits only, because bash re-escapes quotes inside `eval '…'`. Two shells with the same command pair with wrappers in start order. Placeholder sessions (pid 0) get no PID or port, only the output-file states.
- A stopped shell (`TaskStop`, `/tasks`, or session exit) is `exited` with no exit code, not `failed`.
- `Stop.background_tasks` only adopts shells (shells the widget missed, such as ones running before it started or ones moved to the background). It never ends a shell missing from the list, because the task notification can arrive after that `Stop`.
- Command and description are clipped to their first line, 200 chars; matching uses that prefix.
- `src/model.ts` not extended; the build type-checks without it and AV-30 adds the type when it renders.
- `source/process.rs` gets its own small ToolHelp parent table instead of reusing `focus.rs::sys::process_table`, which is private to focus.rs, a file outside this ticket.

## Live check (2026-11-14)

Run from an interactive Claude Code session at the owner's request, against `npm run tauri dev`. The log went `python -m http.server 8000=Running`, then `Serving :8000` and stayed there. `sleep 2; exit 3` went `Running`, then `Failed exit=3` and never reverted. No `last=` appeared because that command prints nothing.

Original instruction: run `npm run tauri dev`. In an interactive session, ask for a dev server in the background (for example `npx vite` or `python -m http.server 8000`) and a background build that fails (for example `sleep 2; exit 3`). Within about 5s the widget log's snapshot line should show `{shells: … =Serving :<port>; … =Failed exit=3 last="…"}`. No reinstall is needed: it uses hooks already installed (`PostToolUse`, `UserPromptSubmit`, `Stop`).
