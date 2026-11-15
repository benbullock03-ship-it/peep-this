---
id: AV-40
title: Global hotkey for panel and strip
status: todo
depends: [AV-39]
attended: false
---

# AV-40 — Global hotkey for panel and strip

**What to build:** `Ctrl+Alt+P` toggles panel and strip from any app; the tray keeps the fully hidden switch (spec §12.6).

- [ ] Global shortcut registered through Tauri's global-shortcut plugin, default `Ctrl+Alt+P`, stored in settings so it can be rebound
- [ ] `strip` to `panel` (pinned) and `panel` to `strip`; from `hidden` it shows the panel
- [ ] The hotkey does not take keyboard focus from the app the user is in
- [ ] Tray left click and Show/Hide toggle `hidden` against the last visible mode
- [ ] A registration clash with another app is reported in the tray menu, not silent
- [x] A collapse control in the header does the same as the hotkey for mouse users (built in AV-39 on the owner's ask: `#collapse-btn` calls `switch_mode`)
- [ ] No `Esc` and no click-outside dismissal
- [ ] The plumbing is shared with AV-24, which registers its own key

**Done when:** the owner toggled the widget from inside a terminal and a browser without losing focus in either, and hid and restored it from the tray.

Decide: header collapse control as a visible glyph or the whole title (recommended: a small chevron shown on header hover; the title stays the drag handle).
