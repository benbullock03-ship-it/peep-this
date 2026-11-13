---
id: AV-13
title: Focus a session's window
status: todo
depends: [AV-03]
attended: false
---

# AV-13 — Focus a session's window

**What to build:** `focus.rs` and a `focus_session` command per spec §6.6: walk from the session PID to the first ancestor with a visible top-level window, restore and foreground it.

- [ ] Handles Windows Terminal, conhost, VS Code, Cursor and the Claude desktop app (record which were tested)
- [ ] Works around Windows foreground-lock rules (record the technique used)
- [ ] No window found: the row flashes instead
- [ ] Clicking a row's peep or name calls it

**Done when:** the owner has clicked rows for sessions in at least two different hosts and the right window came forward.
