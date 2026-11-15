---
id: AV-45
title: Light theme for the pixel shell
status: done
depends: [AV-44]
attended: false
---

# AV-45 — Light theme for the pixel shell

**What to build:** The light palette from `design/shell/shell.css` (`:root[data-theme="light"]`) for the whole widget, following the OS setting.

- [x] `data-theme` on `<html>` follows `prefers-color-scheme` and updates live when the OS switches
- [x] Every surface (rows, open row, approval card, settings and trail log, banners) reads colours from the shell tokens; no hard-coded dark colours left
- [x] AV-36 block bars' amber and red stay readable on the light panel
- [x] Optional override in settings: system, dark, light

**Done when:** the owner switched Windows between light and dark and the widget followed, readable in both.

## Decisions taken without a ruling

- The theme choice is stored in the webview's `localStorage` (`peep.theme`), not `settings.json`. It is read synchronously before first paint, so there is no dark flash on a light desktop. It needs no Rust command.
- The hard shadow stays `rgba(0,0,0,0.45)` in light, as in `design/shell/shell.css`.
- Diff and risk-pill colours have no shell token. They are app tokens (`--diff-*`, `--risk-text`) with light values picked to read on `#FFFCF6`. The light `--ps-dim` is `#776F82` and the light `--amber` is the shell's light `--st-waiting`.

## Done 2026-11-15

Built: `src/theme.ts` (`resolveTheme`, `parsePref`, `initTheme` with a live `matchMedia` listener), `src/theme.test.ts`, the light token block and `color-scheme` in `src/styles.css`, the remaining literals moved to tokens or `color-mix`, and the Theme select in `src/settings.ts`.

Checked by Claude with `?demo` in headless Edge, emulating `prefers-color-scheme`: `data-theme` flips live. The list, open row, approval cards (shell and diff) and settings all read in light, and dark is unchanged. The override beats the OS and "Match Windows" goes back to following it.

Live check (Claude, under the owner's delegation of live checks): with `npm run tauri dev` running, I set `AppsUseLightTheme` to 1, broadcast `WM_SETTINGCHANGE "ImmersiveColorSet"`, then set it back to 0. The widget went light and came back dark, readable both ways. WebView2 follows about 3 to 5 seconds after Windows itself.
