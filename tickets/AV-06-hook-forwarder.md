---
id: AV-06
title: Hook forwarder and hook payload mapping
status: todo
depends: [AV-05]
attended: false
---

# AV-06 — Hook forwarder and hook payload mapping

**What to build:** The path from Claude Code hooks to state (spec §6.2): either native HTTP hooks or a tiny `agentview-hook.exe` that reads stdin, POSTs to the endpoint and exits 0 within 300ms, silently, whether or not the widget runs. Plus the mapping from each hook payload to an `Event`.

- [ ] **Verify** first: does Claude Code support `"type": "http"` hooks? Record the answer here. If yes, skip the binary
- [ ] **Verify** and record: event that follows an approved permission prompt; field distinguishing permission vs idle-reminder `Notification`; subagent tool name; whether `StopFailure` exists
- [ ] Forwarder (if needed): separate bin in the cargo workspace, no stdout, hard 300ms timeout, never non-zero exit
- [ ] Captured real payloads for every event saved as anonymised fixtures; mapping tested against them

**Done when:** with hooks added by hand to a test settings file, a live session cycles thinking → working → waiting → success in the widget's logs.

Decide: native HTTP hooks vs forwarder binary (recommended: native if supported).
