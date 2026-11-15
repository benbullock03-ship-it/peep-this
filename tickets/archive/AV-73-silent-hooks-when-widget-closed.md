---
id: AV-73
title: Hooks stay silent when the widget is closed
status: closed
depends: []
attended: false
---

# AV-73 — Silent hooks when the widget is closed

**Owner report 2026-11-15:** with the widget closed, every session shows `Stop hook error: connect ECONNREFUSED 127.0.0.1:47821`. Hooks are `type: "http"`, and Claude Code reports every refused connection.

**What to build:** hooks become `type: "command"` hooks run through the relay exe the install already copies (`peepthis-statusline.exe`, a new `hook <Event>` mode). It reads stdin, POSTs to `/hook/<Event>` with the token from `endpoint.json`, prints the response body (the `PermissionRequest` decision) and always exits 0. Widget closed: no output, no error.

- [ ] Relay `hook <Event>` mode; the `PermissionRequest` read waits for the broker's hold, the rest keep short timeouts
- [ ] Install writes command hooks; uninstall and stale-entry detection recognise both the old http entries and the new command ones, so an upgrade replaces them cleanly
- [ ] The token leaves `settings.json`
- [ ] Tests: the relay is silent with no endpoint or a refused port, passes a 200 body through, and an install round trip from the old http hooks

**Done when:** with the widget closed, a session's hooks print nothing, and with it open, states and inline approvals work as before.

## Decisions taken without a ruling

- Hook command is shell form `"<relay>" hook <Event>`, quoted like the statusLine; it relies on Git Bash, as the statusLine already does. Without Git Bash Claude Code falls back to PowerShell, where a quoted path with spaces does not run; the `args` exec form would avoid that, but no minimum version is documented, so it was not used.
- Relay read timeout for `PermissionRequest` is 70s, a duplicated constant (hold 60s, under the 75s hook timeout); the bin cannot `#[path]`-include `approve.rs`.
- The install no longer refuses when the server fell back to a random port: the relay reads the live port from `endpoint.json`. `integration_status` no longer takes the endpoint.
- `SessionStart` stays uninstalled even though command hooks allow it; the sessions directory still covers it.
