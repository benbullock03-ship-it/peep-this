---
id: AV-48
title: Peep as the app icon
status: in-progress
depends: []
attended: true
---

# AV-48 — Peep as the app icon

**What to build:** `src-tauri/icons/` still holds the default Tauri logo, so the exe, taskbar, installer and Start menu entry show it. Replace it with a peep.

- [x] Owner picks the peep: colour, accessory, state frame (recommended: the yellow antenna peep already in `icons/tray.png`, which the owner's desktop shortcut uses)
- [x] Render from `design/peep.js` unchanged; whole-pixel scaling only, no redraw
- [x] Regenerate every file in `src-tauri/icons/` (`npm run tauri icon` from a 1024px source), keeping `tray.png`
- [ ] Release build, install, and look at the exe, taskbar, Start menu and installer icons at 100% and 150%

**Done when:** the owner saw the peep on the installed app's exe, taskbar and Start menu entry.

## Built 2026-11-15

Owner ruling 2026-11-15: the yellow antenna peep from `icons/tray.png`. The source is that file scaled 32× nearest-neighbour to 1024px, so it is the same `peep.js` frame and nothing was redrawn. `npm run tauri icon` made the set; `tray.png` is byte-identical to before.

## Decisions taken without a ruling

- `tauri icon` smooths when it shrinks, so the files Windows uses were rewritten as exact whole-pixel scalings: `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, and `icon.ico` with 32, 64, 128 and 256px. The ico has no 16, 24 or 48px entry because 32 does not scale to them in whole pixels; Windows shrinks the 32 for those. If small icons look rough, add smoothed 16/24/48 entries.
- `icon.icns` and the `Square*`/`StoreLogo` files are `tauri icon`'s smoothed output; the NSIS build does not use them.
- The `android/`, `ios/` and `64x64.png` outputs were not kept.

## Owed before done

- Owner: look at the exe, taskbar, Start menu and installer icons at 100% and 150% on the reinstalled build.

Note: the desktop shortcut made 2026-11-15 points at `%APPDATA%\PeepThis\shortcut.ico`, a stand-in outside the repo. Once this lands the shortcut can use the exe's own icon and that file can go.
