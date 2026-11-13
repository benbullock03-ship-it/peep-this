---
id: AV-14
title: Toast and row pulse when a session needs you
status: in-progress
depends: [AV-06, AV-13, AV-04]
attended: false
---

# AV-14 — Toast and row pulse when a session needs you

**What to build:** Alerts per spec §3.6: Windows toast on entering `waiting` with the notification message, click focuses the session; row pulses until the state changes.

- [x] De-duplication: no toast within 30s for the same session, none when the widget is focused and the row visible
- [x] Toast click routes to `focus_session`
- [x] Pulse is subtle (amber glow, ~1.5s), stops immediately on state change
- [x] Unit test for the de-duplication rules

**Done when:** the owner got a toast from a real permission prompt, clicked it, and landed in the right window.

## Decisions taken without a ruling

- Toasts use `tauri-winrt-notification` 0.8.1 (Windows-only dep, same `windows` 0.62 already in the tree) for its in-process `on_activated` click callback; the Tauri notification plugin has no reliable click event on Windows desktop.
- AUMID: release builds use the bundle identifier `com.peepthis.widget` (the installer's Start-menu shortcut carries it); debug builds use PowerShell's AUMID so toasts show under `tauri dev`. Dev toasts are therefore branded "Windows PowerShell".
- Toast click works while the toast is live (banner or Action Center while the widget runs); clicking a stale Action Center entry after the widget exited does nothing (no COM activator registered).
- Row visibility: the backend treats every row as visible when the window holds all content (no scrolling); if content exceeds the 70% height cap the list may be scrolled, so the toast always fires. No frontend reporting, so no render race.
- "Widget focused" is tracked from Tauri `WindowEvent::Focused` in `window.rs`; suppression needs focused AND not `window::is_hidden()` AND not overflowing.
- The 30s window counts only toasts actually shown; an alert suppressed because the widget was in view does not restart it.
- Re-entering `waiting` is detected by comparing each session before and after every store mutation (`notify::entered_waiting`); a second Notification while already waiting does not toast.
- Toast body is the session's task text on entering `waiting` (the notification message, or "Needs permission: <Tool> <target>" when `PermissionRequest` arrived first).
- Pulse is pure CSS off `data-state="waiting"` on live rows (inset amber box-shadow, 1.5s); under `prefers-reduced-motion` it is a static amber outline instead.

## Owed before done

- Owner check in `npm run tauri dev` (toasts show as "Windows PowerShell" in dev): with the widget unfocused, trigger a real permission prompt; a toast appears with session name and message; clicking it lands in that session's terminal. A second prompt within 30s gives no toast. With the widget focused and all rows fitting, no toast but the row pulses; the pulse stops when the prompt is answered.
- Unverified: whether the click handler survives after the toast thread exits (handler is registered on a `ToastNotification` the thread drops), and whether Windows grants foreground on toast click. If clicks do nothing, keep the toast object alive until dismissed.
