---
id: AV-13
title: Focus a session's window
status: done
depends: [AV-03]
attended: false
---

# AV-13 — Focus a session's window

**What to build:** `focus.rs` and a `focus_session` command per spec §6.6: walk from the session PID to the first ancestor with a visible top-level window, restore and foreground it.

- [ ] Handles Windows Terminal, conhost, VS Code, Cursor and the Claude desktop app (record which were tested)
- [x] Works around Windows foreground-lock rules (record the technique used)
- [x] No window found: the row flashes instead
- [x] Clicking a row's peep or name calls it

**Done when:** the owner has clicked rows for sessions in at least two different hosts and the right window came forward.

## Decisions taken without a ruling

- **Foreground lock:** plain `SetForegroundWindow` first (the widget is the foreground process right after the click, so Windows allows it); if the window still is not foreground, a synthetic Alt tap via `SendInput` counts as fresh input and the call is retried with `BringWindowToTop`. Chosen over `AttachThreadInput`, which can hang on a frozen foreground thread. Minimised windows get `SW_RESTORE` first.
- **Walk:** Toolhelp32 snapshot, nearest ancestor first, max depth 64, cycle guard, and stop when a parent's creation time is later than its child's (reused PID). Stops at `explorer.exe`/service hosts so the desktop or taskbar is never "found". Candidate windows: visible, unowned, not `WS_EX_TOOLWINDOW`, non-empty title.
- **Several windows in one process** (VS Code/Cursor run every window from one main process): prefer the window whose title contains the cwd's folder name, else the frontmost.
- **Windows Terminal handoff:** when the shell was handed to WT as default terminal, WindowsTerminal.exe is not an ancestor and the shell's console window is hidden. Fallback: if no ancestor owns a window, focus the frontmost `WindowsTerminal.exe` window. Limitation: with several WT windows this may pick the wrong one; tabs are never targeted (spec §6.6).
- **Hosts verified by window resolution** (`cargo test focus_live -- --ignored --nocapture`, no focus stolen): VS Code (5 live sessions: claude <- powershell <- Code ptyhost <- Code main, resolved to the right VS Code window); conhost (`conhost cmd`: console window is reported as owned by the shell, resolved); WT default-terminal handoff (resolved via the fallback). Not verified: WT launched directly (expected: OpenConsole <- WindowsTerminal is in the chain), Cursor (same Electron shape as VS Code), Claude desktop (none running).
- A failed foreground (`Err`) also flashes the row, same as no window.
- Dead sessions return `false` (flash) without walking.

## Done 2026-11-13

Owner confirmed clicking peep or name brought the right window forward in two different hosts. They didn't name the hosts, so the first box stays unticked: the other hosts in it are untested. The check that was owed:

- Owner check: click rows for sessions in at least two hosts (e.g. VS Code and Windows Terminal) and confirm the right window comes forward; a miss flashes the row. `cargo test focus` passes (8 + 1 live ignored).
