---
id: AV-45
title: Light theme for the pixel shell
status: todo
depends: [AV-44]
attended: false
---

# AV-45 — Light theme for the pixel shell

**What to build:** The light palette from `design/shell/shell.css` (`:root[data-theme="light"]`) for the whole widget, following the OS setting.

- [ ] `data-theme` on `<html>` follows `prefers-color-scheme` and updates live when the OS switches
- [ ] Every surface (rows, open row, approval card, settings and trail log, banners) reads colours from the shell tokens; no hard-coded dark colours left
- [ ] AV-36 block bars' amber and red stay readable on the light panel
- [ ] Optional override in settings: system, dark, light

**Done when:** the owner switched Windows between light and dark and the widget followed, readable in both.
