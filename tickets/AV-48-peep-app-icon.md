---
id: AV-48
title: Peep as the app icon
status: todo
depends: []
attended: true
---

# AV-48 — Peep as the app icon

**What to build:** `src-tauri/icons/` still holds the default Tauri logo, so the exe, taskbar, installer and Start menu entry show it. Replace it with a peep.

- [ ] Owner picks the peep: colour, accessory, state frame (recommended: the yellow antenna peep already in `icons/tray.png`, which the owner's desktop shortcut uses)
- [ ] Render from `design/peep.js` unchanged; whole-pixel scaling only, no redraw
- [ ] Regenerate every file in `src-tauri/icons/` (`npm run tauri icon` from a 1024px source), keeping `tray.png`
- [ ] Release build, install, and look at the exe, taskbar, Start menu and installer icons at 100% and 150%

**Done when:** the owner saw the peep on the installed app's exe, taskbar and Start menu entry.

Note: the desktop shortcut made 2026-11-15 points at `%APPDATA%\PeepThis\shortcut.ico`, a stand-in outside the repo. Once this lands the shortcut can use the exe's own icon and that file can go.
