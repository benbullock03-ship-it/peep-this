---
id: AV-02
title: Session model and pure state engine
status: done
depends: [AV-01]
attended: false
---

# AV-02 — Session model and pure state engine

**What to build:** `src-tauri/src/state.rs` with `Session`, `State` (peep's seven), `Event` (hook events, session-file status, transcript facts, `Tick`) and `fn apply(session, event, now) -> Session`, implementing spec §4 exactly. No I/O.

- [x] Types match spec §5.3 (serde, camelCase) so the snapshot serialises straight to the frontend
- [x] Every row of the §4 table and every rule below it has a unit test
- [x] Stuck: 6 min without an event while `thinking`/`working`, via `Tick`
- [x] `success` held until `UserPromptSubmit`; idle-reminder `Notification` does not override it
- [x] Subagent count increments/decrements/clamps/resets
- [x] Task text per state; `working` shows `<Tool> <target>`, target truncated to 60 chars
- [x] Hook-less fallback: session-file `busy`/`idle` maps to `working`/`idle`

**Done when:** `cargo test state` passes with the cases above.

Decide: whether hook-derived state should ever be overridden by the session file's `status` (recommended: only when no hook event arrived in the last 10s).

## Decisions taken without a ruling

- Session-file `status` applies only when no hook event arrived in the last 10s (recommended option); `busy` lifts idle/success/error to `working`, `idle` drops thinking/working/stuck to `idle`; it never overrides `waiting`, and `idle` never overrides `success`/`error`.
- Time is `now: u64` epoch ms; no new dependencies (serde/serde_json were already present).
- Stuck timer resets on hook and transcript events, not on `Tick`, statusline payloads, or a session-file status that changes nothing (a heartbeat-like `updatedAt` would otherwise mask a hang).
- An event with no transition of its own (`SubagentStop`, idle-reminder `Notification`) ends `stuck` by returning to the pre-stuck state (`thinking` or `working`).
- `success` holds against tool events, `SubagentStop`, `Tick` and the session file; a non-idle `Notification` (permission prompt) still enters `waiting`, `StopFailure` enters `error`, `SessionStart` enters `idle`.
- Idle-reminder `Notification` never changes state in any state, not just `success`.
- Target truncation: over 60 chars becomes the first 59 chars plus `…` (60 chars total, counted in chars not bytes); `lastTool.target` stores the truncated form too, per spec §6.5's no-raw-input rule.
- `working` with no known tool (hook-less `busy`) shows "Working"; a tool without a target shows just the tool name; empty notification message shows "Waiting for input"; empty error summary shows "Error".
- Transcript tool-use/tool-result events are ignored while hooks are fresh (10s); transcript API errors always apply, since `StopFailure` may not exist.
- `stateSince` moves only on a state change (a new tool while already `working` keeps it).
- One `Ended` event covers both PID death and the `SessionEnd` hook: state becomes `success` if it was `success`, else `idle`; `alive=false`, `endedAt=now`; all later events are ignored.
- Internal bookkeeping (`lastEventAt`, `lastHookAt`, `preStuck`) lives on `Session` with `#[serde(skip)]` so the snapshot stays the §5.3 shape.
