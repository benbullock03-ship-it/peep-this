---
id: AV-20
title: Verify PermissionRequest decisions over HTTP hooks
status: todo
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
