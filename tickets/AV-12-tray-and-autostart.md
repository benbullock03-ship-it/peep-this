---
id: AV-12
title: Tray icon and start at login
status: todo
depends: [AV-11]
attended: false
---

# AV-12 — Tray icon and start at login

**What to build:** Tray icon (left click toggles the widget; menu: Show/Hide, Start at login, Quit) and autostart via the Tauri autostart plugin, on by default. AV-08 adds its install items to this menu.

- [ ] Tray menu has Install integration / Uninstall integration items calling the `install_integration` / `uninstall_integration` commands from AV-08 (then drop the header right-click stopgap in `src/main.ts`)
- [ ] Hiding the window pauses peep rendering (no rAF work while hidden)
- [ ] Single instance: a second launch shows the existing widget
- [ ] Tray icon uses a peep frame (ask the owner for a 32px export if a static frame from `peep.js` does not read at tray size)

**Done when:** the owner has toggled it from the tray and seen it start after a sign-out/in.
