---
id: AV-06
title: Native HTTP hooks and hook payload mapping
status: in-progress
depends: [AV-05]
attended: false
---

# AV-06 — Native HTTP hooks and hook payload mapping

**What to build:** The path from Claude Code hooks to state (spec §6.2): either native HTTP hooks or a tiny `peepthis-hook.exe` that reads stdin, POSTs to the endpoint and exits 0 within 300ms, silently, whether or not the widget runs. Plus the mapping from each hook payload to an `Event`.

- [x] **Verify** first: does Claude Code support `"type": "http"` hooks? Record the answer here. If yes, skip the binary
- [x] **Verify** and record: event that follows an approved permission prompt; field distinguishing permission vs idle-reminder `Notification`; subagent tool name; whether `StopFailure` exists
- [x] Forwarder (if needed): not needed
- [ ] Captured real payloads for every event saved as anonymised fixtures; mapping tested against them (all except `Notification`, see Owed)

**Done when:** with hooks added by hand to a test settings file, a live session cycles thinking → working → waiting → success in the widget's logs.

Decide: native HTTP hooks vs forwarder binary (recommended: native if supported).

## Verified 2026-11-13 (Claude Code 2.1.323)

Sources: the hooks docs, plus real payloads from `claude -p --settings <file>` with `http` hooks pointed at a capture server.

- **HTTP hooks: supported.** `{"type":"http","url":...,"timeout":N,"headers":{...}}`. The hook JSON is the POST body. An empty 2xx means no output and no decision. Unreachable, non-2xx and timeout are non-blocking errors. The default timeout is 600s, so we set 1. Header values interpolate only env vars listed in `allowedEnvVars`, which the widget cannot set, so the token goes in literally. `--settings <file>` adds hooks to a session.
- **`SessionStart` rejects `http`** (command and mcp_tool only). Not installed; the sessions directory covers it.
- **Permission order:** `PreToolUse`, then `PermissionRequest` while the prompt is up, then `PostToolUse` after approval and execution. `PermissionRequest` has `tool_name` and `tool_input` but no `tool_use_id`. It fires in `-p` too, where the request is auto-denied and `Stop` follows. `PermissionDenied` is for auto-mode classifier denials only.
- **Notification kinds:** `notification_type` with values `permission_prompt`, `idle_prompt`, `elicitation_dialog`, `agent_needs_input` and others (docs). The mapping is unchanged: only `idle_prompt` is idle.
- **Subagent tool:** `Agent`, with `tool_input.description` and `subagent_type`. `SubagentStart` and `SubagentStop` carry `agent_id` and `agent_type`. A subagent's own tool events arrive under the parent `session_id` with `agent_id` set. Its hand-back also fires a parent `UserPromptSubmit`.
- **`StopFailure`: exists**, and fires instead of `Stop`. `error` is a type code (`model_not_found`, `rate_limit`, `overloaded`, ...), `error_details` is optional prose, and `last_assistant_message` holds the shown text.
- `PostToolUseFailure` has `error` ("Exit code 3") and `is_interrupt`. `SessionEnd.reason` is `other` in `-p`.

## Decisions taken without a ruling

- Native HTTP hooks; no forwarder binary. Entries come from `ingest/hooks.rs::settings_hooks(port, token)` for AV-08: 10 events, `timeout: 1`, literal `X-PeepThis-Token` header.
- The token now persists: `spawn` reuses a valid 64-hex token from `endpoint.json` and generates one only when it is missing or invalid. Otherwise every app restart would 401 the installed hooks. This changes the AV-05 "per app start" rule.
- Port fallback is kept, but hooks hold a fixed URL, so on the fallback port they miss the widget (non-blocking). The fallback log line now says so. AV-08's `outdated` check should compare the installed port and token with `endpoint.json`.
- `PermissionRequest` maps to `Notification { idle: false }` with message `Needs permission: <Tool> <target>`. A later `permission_prompt` Notification overwrites the text with Claude's wording; the state is the same.
- `StopFailure` summary: `API error: <error_details>`, else `API error: <error with _ as spaces>`. `last_assistant_message` is never read.
- `SubagentStart` is not mapped: the count still comes from `PreToolUse Agent`. Subagent tool events drive the parent row's task like any tool, and that is acceptable.
- The snapshot log line now shows `name=State` per live session, so the cycle can be read from the logs.
- Spec §4, §5, §6.2, §6.5 and §7 updated to match.

## Live check 2026-11-13

`npm run tauri dev`, then `claude -p "... echo hi, then write perm2.txt ..." --settings <test settings from settings_hooks shape> --allowedTools Bash --permission-mode default`. Widget log for that session: `Thinking` → `Working` → `Waiting` (PermissionRequest) → `Success` (Stop) → ended (SessionEnd). Done-when is met.

## Owed before done

- A real `Notification` payload: it fires only in interactive sessions. `notification_permission.json` and `notification_idle.json` are still synthetic, though they follow the documented field names. Owner: start an interactive `claude --settings <hooks file>`, trigger a permission prompt, then let it sit idle. Confirm the row goes `Waiting`, then `Success` after the idle reminder, with no toast regression. Capture the body if you can (scratch capture server: `node server.js <outdir> 47999`).
