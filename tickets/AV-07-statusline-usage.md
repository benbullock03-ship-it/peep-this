---
id: AV-07
title: Statusline wrapper, context bars and rate-limit header
status: todo
depends: [AV-05, AV-04]
attended: false
---

# AV-07 — Statusline wrapper, context bars and rate-limit header

**What to build:** `agentview-statusline.exe` per spec §6.3, and the UI that shows its data: per-row context bar and the header's 5h/7d rate limits (spec §3.2).

- [ ] Wrapper runs the recorded original command with identical stdin and prints its stdout byte-for-byte
- [ ] POST is fire-and-forget; overhead measured and recorded here (budget under 50ms)
- [ ] Works when the original is a `sh` script (the owner's current statusline is `statusline-command.sh` run by Git Bash)
- [ ] Context % = 100 − `remaining_percentage`; rate limits from the latest payload of any session, age on hover
- [ ] Header and bars hidden until data arrives

**Done when:** with the wrapper configured by hand, the owner's statusline looks unchanged and the widget shows context and rate limits.
