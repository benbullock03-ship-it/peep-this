---
id: AV-12
title: Tray icon and start at login
status: in-progress
depends: [AV-11]
attended: false
---

# AV-12 — Tray icon and start at login

**What to build:** Tray icon (left click toggles the widget; menu: Show/Hide, Start at login, Quit) and autostart via the Tauri autostart plugin, on by default. AV-08 adds its install items to this menu.

- [x] Tray menu has Install integration / Uninstall integration items calling the `install_integration` / `uninstall_integration` commands from AV-08 (then drop the header right-click stopgap in `src/main.ts`)
- [x] Hiding the window pauses peep rendering (no rAF work while hidden)
- [x] Single instance: a second launch shows the existing widget
- [x] Tray icon uses a peep frame (ask the owner for a 32px export if a static frame from `peep.js` does not read at tray size)

**Done when:** the owner has toggled it from the tray and seen it start after a sign-out/in.

## Built 2026-11-13

`src-tauri/src/tray.rs` (menu, left-click toggle, start at login), `window::set_hidden` / `is_hidden`, and plugins `tauri-plugin-single-instance` and `tauri-plugin-autostart` in `lib.rs`. `Integrator::install` / `uninstall` back both the tray and the commands. In the frontend, `src/banner.ts` loses its disconnect mode and the right-click stopgap is gone.

## Decisions taken without a ruling

- **Pausing peeps:** on `widget-hidden`, the frontend detaches `#sessions` from the DOM. peep.js's shared loop stops when no `<peep-bot>` is connected, so a hidden widget does zero rAF work. Rows keep updating while detached, and reattaching continues their animations rather than restarting them. Content-height fits are skipped while detached, so the window keeps its size.
- **Hidden means hidden:** `fit_window` and the 2s show fallback no longer re-show a window the user hid from the tray. Without this, any new session would pop it back.
- **Tray icon:** `src-tauri/icons/tray.png`, a 32px idle yellow peep. It is a frame of the owner's `peep.js`, rendered at 128px in headless Edge and reduced 4× nearest-neighbour, so the pixels are exact and nothing was redrawn. It is static, not animated.
- **Start at login:** turned on by default the first time a release build runs. A marker file `%APPDATA%\PeepThis\autostart-default-applied` makes a later "off" stick. Debug builds skip the default so `tauri dev` never registers the dev exe; the menu toggle still works there.
- **Tray install/uninstall:** runs off the event loop. The result goes to the frontend as an `integration-result` event. A failure shows the widget with the error in the banner. A successful uninstall marks the install banner dismissed, so it does not nag straight after.
- **Menu order:** Show/Hide | Install integration, Uninstall integration | Start at login | Quit. Both integration items are always enabled; uninstall when nothing is installed is a no-op.

## Owed before done

Owner, in `npm run tauri dev`:
1. The tray shows the peep icon. Left click hides/shows the widget, and so does Show/Hide.
2. While hidden, Task Manager shows the WebView2 process near 0% CPU.
3. Run `src-tauri\target\debug\peep-this.exe` a second time: no new window, and the existing one shows.
4. Install integration / Uninstall integration work from the tray (AV-08's checks).
5. Start at login: build a release (`npm run tauri build`, local only), run it once, sign out and back in, and the widget starts.
