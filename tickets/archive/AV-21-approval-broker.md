---
id: AV-21
title: Approval broker: hold, decide, release
status: done
depends: [AV-20]
attended: false
---

# AV-21 — Approval broker: hold, decide, release

**What to build:** The backend half of inline approvals (spec §11.1). The event server stops answering `PermissionRequest` with an instant 204. It holds the connection, exposes the request as a pending approval in the snapshot, and answers with the user's decision or releases to the terminal.

- [x] New `approve.rs`: pending map keyed by a widget-generated request id, one held responder per request; several per session allowed (subagents)
- [x] Tauri command `decide_approval(requestId, allow | deny, reason?)`. Decisions come only from this command. No HTTP route accepts a decision
- [x] Release with no decision (the contract AV-20 recorded) when: the hold window ends (default 60s, setting), the session's terminal becomes the foreground window, the session leaves `waiting` by another event, the session dies, or the client disconnects
- [x] Foreground check reuses `focus.rs` PID-to-window resolution; polls only while something is pending
- [x] `settings_hooks` gives `PermissionRequest` a timeout above the hold window; other events stay at 1. Installed entries with the old timeout report `integration: "outdated"`
- [x] Snapshot: `Session.pending: PendingApproval[]` with id, tool, full input needed by the card (command, file path, old/new strings), cwd, agent id, received-at, release-at. Held in memory only while pending
- [x] Widget hidden or quitting: release everything at once
- [x] Tests from AV-20 fixtures: allow, deny with reason, timeout release, focus release, session death, double decide, decide after release

**Done when:** with the card stubbed by a dev-only command, a live session's tool call was allowed and another denied from the widget process, and an ignored one fell through to the terminal prompt after the window.

Decide: hold window default (recommended: 60s). Decide: a hung widget can stall a prompt for up to the hook timeout (recommended: accept, keep timeout at hold + 15s, document it).

## Done 2026-11-14

- `src-tauri/src/approve.rs`: `Broker` holds each `PermissionRequest` with its socket (`TcpResponder`) and puts a `PendingApproval` (id, tool, raw `tool_input`, cwd, agent id, received/release time, plus the AV-22 risk `Verdict`) into `Session.pending` via `Store::set_pending`. A sweep thread runs every 250ms only while something is held or awaiting settlement. It releases (204) on: window end, terminal foreground (`focus::terminal_is_foreground`, same resolution as focus), the session or the request's own agent leaving `waiting`, session death, and widget hidden. A peer disconnect drops the request without a reply. `RunEvent::Exit` releases everything. `decide_approval` is the only decision path.
- `src-tauri/src/ingest/http.rs`: tiny_http replaced by a small std HTTP/1.1 reader/writer (Content-Length and chunked, 16KB head cap, 1MB body cap, one request per connection, thread per connection). `server::handle_full` returns `Outcome::Hold` for a PermissionRequest; the connection goes to the broker.
- `state.rs`: `Event::Approval { agent_id, outcome }`. Allowed goes to `working`, Denied to `thinking`, and Abandoned (the terminal took it and no `PostToolUse` came within 3s) to `idle`. For a subagent this acts on the agent and releases the parent's agent wait.
- `hooks.rs`: PermissionRequest timeout 75 (hold + 15), others 1; `install::integration_status` compares timeouts, so existing installs read `outdated`.
- Dev-only card stub: `src/devApprovals.ts`, rendered in `import.meta.env.DEV` builds under the list, with Allow/Deny buttons calling `decide_approval`.
- Tests: `cargo test approve` (16: snapshot shape, allow, deny with reason, default reason, contract bodies vs the AV-20 fixtures, window, foreground, hidden, session death, left waiting, quit, double decide, decide after release, terminal-no then idle, terminal-yes left to PostToolUse, subagent isolation and resume), `ingest::http` (5), and two TCP tests in `ingest::server` (held until decided while other hooks still pass; dropped connection seen and released). Full run: 229 Rust + 36 Vitest passing, `npm run build` clean.
- **Live (done-when):** `npm run tauri dev` with WebView2 remote debugging. A real interactive `claude` (PTY driver, launched windowless through WMI, `--settings` pointing at the widget's port and token with timeout 75) asked for a Write each time, and decisions went through the widget's own `decide_approval` (CDP `Runtime.evaluate` of `__TAURI__.core.invoke`). Allow: `allowed (delivered: true)`, file written. Deny: Claude showed `Denied by PermissionRequest hook` and quoted the reason. Ignored: held 60s with the terminal prompt on screen, `released approval … (Window)`, then the prompt still worked and Yes wrote the file. The owner also clicked Allow on the dev stub during a run, and it went through.
- First live attempt released at once as `Foreground`: a console window that WMI gave the driver's `python.exe` took focus, and it sat in the session's process chain. That is the foreground rule working; the harness then used `pythonw.exe`.

## Decisions taken without a ruling

- Hold window 60s (`approve::HOLD_MS`); hook timeout 75s (hold + 15). A hung widget can delay a prompt by up to 75s, but the terminal prompt is visible and answerable the whole time (AV-20), so it is accepted. The "setting" is a constant for now: there is no widget settings store, and changing the hold means reinstalling hooks so the timeout follows.
- tiny_http removed in favour of `ingest/http.rs`, because holding needs the raw socket to see Claude Code drop the connection. No new crate.
- A subagent's request follows that agent's state, not the parent's, so deciding one request does not release a sibling's.
- Terminal-answered requests (connection dropped): no reply. After 3s with the session or agent still in the same `waiting`, it goes `idle` (AV-20: no `Stop` follows a terminal "No"). A terminal "Yes" shows up as `PostToolUse` first.
- Allow moves the row to `working` at once rather than waiting for `PostToolUse`, so the card does not leave a stale `waiting` behind.
- Foreground uses `focus.rs` resolution as is, including its frontmost-Windows-Terminal fallback. With several WT windows, any foreground WT window can count as the session's terminal. Accepted, since tabs cannot be told apart anyway.
- A held request is not sent the trailing `permission_prompt` Notification alert twice: unchanged, because `notify` keeps the existing waiting task (AV-23/AV-25 own the toast).
- `PendingApproval` also carries the AV-22 risk verdict, since every surface needs it.

## Owed (not blocking done)

- The owner's installed hooks still have timeout 1, so the widget shows `outdated`, and held requests fall through to the terminal after 1s until **Install integration** is run again.
