---
id: AV-21
title: Approval broker: hold, decide, release
status: todo
depends: [AV-20]
attended: false
---

# AV-21 — Approval broker: hold, decide, release

**What to build:** The backend half of inline approvals (spec §11.1). The event server stops answering `PermissionRequest` with an instant 204. It holds the connection, exposes the request as a pending approval in the snapshot, and answers with the user's decision or releases to the terminal.

- [ ] New `approve.rs`: pending map keyed by a widget-generated request id, one held responder per request; several per session allowed (subagents)
- [ ] Tauri command `decide_approval(requestId, allow | deny, reason?)`. Decisions come only from this command. No HTTP route accepts a decision
- [ ] Release with no decision (the contract AV-20 recorded) when: the hold window ends (default 60s, setting), the session's terminal becomes the foreground window, the session leaves `waiting` by another event, the session dies, or the client disconnects
- [ ] Foreground check reuses `focus.rs` PID-to-window resolution; polls only while something is pending
- [ ] `settings_hooks` gives `PermissionRequest` a timeout above the hold window; other events stay at 1. Installed entries with the old timeout report `integration: "outdated"`
- [ ] Snapshot: `Session.pending: PendingApproval[]` with id, tool, full input needed by the card (command, file path, old/new strings), cwd, agent id, received-at, release-at. Held in memory only while pending
- [ ] Widget hidden or quitting: release everything at once
- [ ] Tests from AV-20 fixtures: allow, deny with reason, timeout release, focus release, session death, double decide, decide after release

**Done when:** with the card stubbed by a dev-only command, a live session's tool call was allowed and another denied from the widget process, and an ignored one fell through to the terminal prompt after the window.

Decide: hold window default (recommended: 60s). Decide: a hung widget can stall a prompt for up to the hook timeout (recommended: accept, keep timeout at hold + 15s, document it).
