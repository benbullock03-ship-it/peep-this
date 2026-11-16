---
id: AV-111
title: Focus-aware toasts and an optional alert sound
status: open
depends: []
---

# AV-111 — Focus-aware toasts and an optional alert sound

**What to build:** Two alert fixes competitors' users ask for most.

- [ ] No toast when the session's own terminal window (the one `focus_session` would raise) is already foreground; the row still pulses
- [ ] Optional sound on entering `waiting` or `error`, off by default, toggled in settings; never for subagent completion or `success`
- [ ] Rust test for the foreground check against a fake window resolver; update spec §7

**Done when:** with the session's terminal focused no toast appears, with it unfocused one does, and the sound plays only when switched on; the owner confirms.
