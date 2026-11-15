---
id: AV-40
title: Global hotkey for panel and strip
status: in-progress
depends: [AV-39]
attended: false
---

# AV-40 — Global hotkey for panel and strip

**What to build:** `Ctrl+Alt+P` toggles panel and strip from any app; the tray keeps the fully hidden switch (spec §12.6).

- [x] Global shortcut registered through Tauri's global-shortcut plugin, default `Ctrl+Alt+P`, stored in settings so it can be rebound
- [x] `strip` to `panel` (pinned) and `panel` to `strip`; from `hidden` it shows the panel
- [x] The hotkey does not take keyboard focus from the app the user is in
- [x] Tray left click and Show/Hide toggle `hidden` against the last visible mode
- [x] A registration clash with another app is reported in the tray menu, not silent
- [x] A collapse control in the header does the same as the hotkey for mouse users (built in AV-39 on the owner's ask: `#collapse-btn` calls `switch_mode`)
- [x] No `Esc` and no click-outside dismissal
- [x] The plumbing is shared with AV-24, which registers its own key

**Done when:** the owner toggled the widget from inside a terminal and a browser without losing focus in either, and hid and restored it from the tray.

Decide: header collapse control as a visible glyph or the whole title (recommended: a small chevron shown on header hover; the title stays the drag handle).

## Decisions taken without a ruling

- Header collapse control: left as AV-39 built it at the owner's ask (an always-visible `›` button, `#collapse-btn`), not the hover-only chevron. The owner saw that during AV-39; its tooltip now names the live hotkey.
- The two hotkeys are slots in `hotkey.rs` on one plugin handler. `Ctrl+Alt+P` is no longer hard-reserved; each slot instead refuses the other's current combination.
- The toggle never takes focus, including when it brings the widget back from `hidden`. Tray Show/Hide and single-instance relaunch still focus the widget, as before.
- Clash report: a clickable line at the top of the tray menu ("Ctrl+Alt+P is taken by another app: change it") that opens settings. Settings gets a "Panel/strip hotkey" field next to the inbox one, and the panel shows a warning notice like the inbox's.
- In a browser (`npm run dev`) `Ctrl+Alt+P` toggles locally while the page has focus.

## Owed before done

Live check 2026-11-15 (`npm run tauri dev`, keys sent with `keybd_event`, foreground read with `GetForegroundWindow`, shots in the session scratchpad `AV-40/`):

- Seen: with Notepad in front, 16 presses alternated strip and panel and the foreground stayed Notepad every time. Hidden from the tray, then the hotkey: the panel came back and Notepad kept the foreground. Tray click hid it, and the next tray click brought back the strip (the last visible mode). With another process holding `Ctrl+Alt+P` at launch, the tray menu showed the clash line at the top. Clicking it opened settings, which showed the error under the Panel/strip field.
- Odd once: in the first dev run, three presses over Notepad did nothing after two that had worked. There was no trace in that run. After a restart with a trace, every press arrived (18 of 18), including the same Alt-tap sequence. Not reproduced.
- Side effect seen: Windows 11 Notepad shows its Alt keytips after the combo, because it sees Alt go up without the P. This comes from the app, not the widget, but the owner may notice it.
- Not seen: a real terminal or browser in front (only Notepad was driven), rebinding through the settings field live (the save and reload are unit-tested), and Firefox, which was not running.
- Owner to look at for "Done when": with the strip showing, press `Ctrl+Alt+P` while typing in a terminal, then in a browser. Each press should switch strip and panel, and the next keystroke should still go to that app. Then hide from the tray icon and click it again: the widget comes back in the same mode it was in.
