---
id: AV-20
title: Verify PermissionRequest decisions over HTTP hooks
status: done
depends: []
attended: false
---

# AV-20 — Verify PermissionRequest decisions over HTTP hooks

**What to build:** Nothing shippable. A capture-server spike (as in AV-06) that proves the widget can answer a permission prompt, and records the exact contract here for AV-21 to build on (spec §11.1).

- [ ] **Verify** the response body that allows and that denies a `PermissionRequest` from an `http` hook; record the exact JSON. Confirm a deny message reaches Claude as feedback
- [ ] **Verify** while the hook response is held: does the terminal show its prompt, and can the user answer there? Record what the session does when the terminal answer and the hook answer race
- [ ] **Verify** an empty 204 after a held 10s, 60s and 120s still makes the normal terminal prompt appear, and nothing is lost
- [ ] **Verify** the per-entry `timeout` can differ between events (PermissionRequest long, the rest stay 1) and what the session does when the timeout fires
- [ ] **Verify** what the hook connection does when the session is killed or the user presses Esc while it is held (does the server see a disconnect?)
- [ ] **Verify** a subagent's permission request: carries `agent_id`? Same contract?
- [ ] **Verify** `PermissionRequest` has anything usable to correlate with the following `PostToolUse` (AV-06 found no `tool_use_id`)
- [ ] Captured payloads and responses saved as anonymised fixtures

**Done when:** a live interactive session had one tool call allowed and one denied with a reason purely by the capture server's response, and every Verify line above has a recorded answer.

Decide: if holding the hook blocks the terminal prompt entirely and a late 204 does not bring it back, the "widget first, then release" model fails. Recommended fallback: hold only while the session's terminal is not the foreground window, and cap the hold at the largest value verified to release cleanly.

## Verified 2026-11-14 (Claude Code 2.1.325)

Sources: the hooks docs (`code.claude.com/docs/en/hooks.md`, PermissionRequest, HTTP response handling, Timeouts, `onFailure`), plus a capture server on 127.0.0.1:47913 driven by `claude -p ... --settings <scratch> --setting-sources project --permission-mode default --model haiku`. Every non-permission event had `timeout: 1`; the server picked its reply from the URL path. The `-p` runs answered everything except the terminal-UI questions.

- **Allow body** (HTTP 200, `application/json`), verified: the Write ran and `PostToolUse` followed.
  `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}`
  Optional `updatedInput` replaces the whole input and `updatedPermissions` applies rule or mode updates; we use neither. An allow does not override a matching deny rule.
- **Deny body**, verified: `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"deny","message":"<reason>"}}}`. The message reaches Claude verbatim as the tool_result (`is_error: true`), and Claude quoted it back. The transcript records `permissionDecision.source: "hook"`. **No `PostToolUse` or `PostToolUseFailure` fires after a hook deny**: the next event was `Stop`.
- **`interrupt: true`** on a deny stops the turn at once. `-p` printed "Execution error" and exited 1. **No `Stop` fired**, only `SessionEnd`.
- **Long holds work.** A hold of 120s ending in an allow (entry `timeout: 200`) was honoured: the write happened after 120s. A held 10s 204 gave "no decision", and `-p` then auto-denied with the standard "you haven't granted it yet" error, which is the normal `-p` flow. Nothing was lost and the session finished cleanly. Whether the terminal prompt shows during and after a hold is owed (below).
- **Per-entry timeouts can differ:** PermissionRequest `5`, the other events `1`. At 5.00s Claude closed the socket (the server saw a disconnect before replying) and treated the result as no decision; `-p` auto-denied and `Stop` followed. Docs: a timed-out http hook is cancelled, its output discarded, and no decision results. **`onFailure: "block"` on PermissionRequest turns a failure or timeout into a deny**, so it must stay unset.
- **Kill while held:** killing the `claude` process 22s into a hold made the server see the connection close at once. No `SessionEnd` was sent. Esc while held is owed.
- **Subagent:** same contract. Its PermissionRequest carries `agent_id` and `agent_type` under the parent `session_id`, and an allow from the server let the subagent's Write run. The surrounding PreToolUse/PostToolUse carry the same `agent_id`.
- **Correlation:** PermissionRequest still has no `tool_use_id` (docs confirm this). It comes about 50ms after a `PreToolUse` with the same `session_id`, `prompt_id`, `agent_id` (or absent), `tool_name` and an identical `tool_input`, and that PreToolUse carries the `tool_use_id` that the following `PostToolUse` repeats. The join is usable, but it rests on inference.
- Docs, not verified live: in SDK hosts (Desktop, VS Code) the hooks run alongside `canUseTool` and "whichever decides first applies". The `permission_prompt` Notification is skipped if "you or a PermissionRequest hook answer sooner", which suggests the dialog and the hook race rather than run in sequence.
- Fixtures: `permission_request_bash.json`, `permission_request_subagent.json`, `permission_response_allow.json`, `permission_response_deny.json` (the existing `permission_request.json` covers Write).

## Decisions taken without a ruling

- The widget sends the minimal allow/deny bodies above. No `updatedInput`, no `updatedPermissions` (spec §11.1 keeps rule-writing out), no `interrupt`: an interrupt sends no `Stop`, so the row would hang in its old state.
- The PermissionRequest entry gets `timeout` = hold window + 5s (65 by default). The other events stay at 1. The widget releases with its own 204 first, and Claude's timeout is only the backstop. Changing the hold window means reinstalling hooks, so AV-08's `outdated` check should compare this timeout too. `onFailure` stays unset.
- A connection that closes before the widget replies (timeout, kill, or probably the terminal answering first) counts as a release: drop the pending card and do not try to reply.
- After the widget denies, the state engine leaves `Waiting` on the widget's own decision (to `thinking`), because no PostToolUse follows. After an allow, the normal PostToolUse arrives.
- Correlation for AV-21/AV-26: the latest unresolved PreToolUse in the same `session_id` + `agent_id` with equal `tool_name` and deep-equal `tool_input` supplies the `tool_use_id`.
- Decide fallback: **not triggered** on what could be verified. A late 204 is a clean "no decision", and holds up to 120s were honoured. The terminal-side behaviour below could still trigger it.

## Verified interactively 2026-11-14 (Claude Code 2.1.325)

Real interactive `claude` (TUI) driven through a pseudo-terminal (pywinpty + pyte screen capture, scratchpad `drive.py`), `--permission-mode default --setting-sources project --model haiku`, against the capture server below. Prompt each time: "use Write to create x.txt containing hi". Note: **auto mode is now the default permission mode**; without `--permission-mode default` the Write ran with no PermissionRequest at all.

- **Terminal prompt during a hold: shown.** The full "Do you want to create x.txt? 1. Yes / 2. … / 3. No" dialog is on screen 4s into every hold (10s, 60s, 120s). The hook does not block it.
- **After a late 204: the prompt stays and works.** After 10s, 60s and 120s holds ending in 204, the same dialog is still up; Enter (Yes) wrote the file and `PostToolUse` followed. Nothing lost.
- **`Notification` `permission_prompt` still fires about 6s into a hold** (10s, 60s, 120s runs), while the hook is unanswered. It is skipped when an answer lands sooner (allow, deny, and the timeout run answered at about 6s).
- **Race, terminal first:** answering **No** or **Esc** in the terminal during a 20s `holdallow` rejected the write ("User rejected write to x.txt"). Claude closed the hook connection within about 1s of the keypress (`closed_before_reply`), and the server's allow at 20s found the socket gone. **The terminal answer wins and the hook answer is discarded.** **No `Stop`, `PostToolUse` or `PostToolUseFailure` fired after the terminal rejection**, although the turn ended on screen ("done"); the next event was `SessionEnd` when the driver quit.
- **Timeout:** with PermissionRequest `timeout: 5` and the server holding 30s, the connection closed at 5.0s, the terminal prompt stayed usable, and Yes wrote the file.
- **Esc / kill while held:** both close the connection at once (`closed_before_reply`). Closing the terminal sent no `SessionEnd`.
- **Done-when:** with `/allow` the write ran without touching the terminal (`PostToolUse`, then `Stop`). With `/deny` the screen showed `Error: Denied from widget: ZEBRA-42 do not write that file` and `Denied by PermissionRequest hook`, Claude quoted ZEBRA-42 back, and `Stop` followed.
- **Decide fallback: not taken.** Widget-first-then-release works: the terminal dialog is live the whole time, and either side can answer.

Decisions for AV-21 from this run:
- A connection closed before the widget replies means the terminal (or a timeout or kill) took over. Drop the card. If no `PostToolUse` for that tool arrives within a few seconds, assume the terminal said no: leave `waiting` for `idle`, since no `Stop` will come.
- Expect a `permission_prompt` `Notification` about 6s into a hold. It must not raise a second toast for a request the widget is already showing.

## Harness

Scratch harness (re-create if the temp dir was cleared): `C:\Users\benbu\AppData\Local\Temp\claude\30a9097ea8fcfb1f\23e4d5f3-9d16-457d-aa31-d86775f82627\scratchpad\cap\`. The server (`server.js`) logs to `log.jsonl` and replies by path: `/allow`, `/deny`, `/hold/<s>` (204 after s), `/holdallow/<s>`. To regenerate settings, run `node gen.js <out> <path> <timeout>` in Git Bash with `MSYS_NO_PATHCONV=1`.

1. `node server.js 47913`. In an empty scratch dir run `claude --settings <cap>\s_hold60.json --setting-sources project` and ask "use Write to create x.txt containing hi". Record whether the terminal permission prompt is visible in the first 5s. Leave it alone: after the 204 at 60s, does the prompt appear (or stay) and does answering Yes write the file? Repeat with `s_hold10.json` and `s_hold120.json`.
2. Race: with `s_holdallow.json` (allows at 20s), answer **No** in the terminal at about 5s. Record which answer wins and whether `log.jsonl` shows `closed_before_reply` right after your keypress. Repeat with Esc instead of No.
3. Timeout: with `s_tmo.json` (timeout 5, server holds 30), confirm the terminal prompt is usable after 5s.
4. Kill: with `s_kill.json`, close the terminal window during the hold and confirm `closed_before_reply` in the log.
5. Done-when: with `s_allow.json` one write goes through without touching the terminal. With `s_deny.json` one is denied, and Claude quotes "ZEBRA-42".

All five steps were run on 2026-11-14 through the PTY driver (`drive.py`, `runall.sh`, `summ.py` in session scratchpad `a2ffea77-…/scratchpad`); results above.
