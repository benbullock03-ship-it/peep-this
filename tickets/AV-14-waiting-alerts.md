---
id: AV-14
title: Toast and row pulse when a session needs you
status: todo
depends: [AV-06, AV-13, AV-04]
attended: false
---

# AV-14 — Toast and row pulse when a session needs you

**What to build:** Alerts per spec §3.6: Windows toast on entering `waiting` with the notification message, click focuses the session; row pulses until the state changes.

- [ ] De-duplication: no toast within 30s for the same session, none when the widget is focused and the row visible
- [ ] Toast click routes to `focus_session`
- [ ] Pulse is subtle (amber glow, ~1.5s), stops immediately on state change
- [ ] Unit test for the de-duplication rules

**Done when:** the owner got a toast from a real permission prompt, clicked it, and landed in the right window.
