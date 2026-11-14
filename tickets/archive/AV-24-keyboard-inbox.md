---
id: AV-24
title: Keyboard approval inbox on a global hotkey
status: done
depends: [AV-23]
attended: false
---

# AV-24 — Keyboard approval inbox on a global hotkey

**What to build:** A global hotkey that brings the oldest pending approval across all sessions front and centre, answerable without the mouse or a window switch (spec §11.4). It reuses AV-23's card.

- [x] Global hotkey registered through Tauri's global-shortcut plugin; configurable; a clash with another app is reported, not silent
- [x] Hotkey shows the widget if hidden, focuses it, and puts the oldest pending card in focus with session name, peep and project colour
- [x] Keys: `Y` allow, `N` deny, `R` deny with reason, `Tab`/`Shift+Tab` next/previous, `Enter` focus that session's terminal, `Esc` dismiss and return focus to the previous window
- [x] `destructive` tier: `Y` must be held for 1s
- [x] After a decision the next pending card takes focus; when none remain, focus returns to the window the user came from
- [x] Hotkey with nothing pending flashes the header and does nothing else
- [x] Header shows a pending count when > 0

**Done when:** the owner cleared three pending approvals from three sessions using only the keyboard and ended up back in the window they started in.

## Done 2026-11-14

Live check by Claude at the owner's request, in `npm run tauri dev`, with real keystrokes (`keybd_event`) and three held requests from three sessions:

- Badge "3", title "3 pending approvals · Ctrl+Alt+I to review".
- `Ctrl+Alt+I` from Notepad raised the widget past the foreground lock and focused the oldest card. Tab / Shift+Tab moved focus.
- Y allowed the safe card. A 300ms Y on the destructive card did nothing, and a 1.3s hold allowed it. R, typed reason and Enter denied with that reason. Hook responses matched.
- Esc partway returned focus to Notepad.
- Enter on a card for this session focused its VS Code window, and the broker released that request at once (terminal focused).
- Emptying the inbox from a VS Code start returned focus to VS Code.
- A press with nothing pending flashed the title (`is-flash` for about 1s) and left the foreground alone.
- Clash on rebind (Ctrl+Alt+K held by another process): settings shows the error and Ctrl+Alt+I stays live.
- Clash at startup (Ctrl+Alt+I held): the amber list notice shows, Change opens settings, and saving Ctrl+Alt+I once free clears it. `inboxHotkey` is now stored in settings.json.

Follow-up, not blocking: the clash message ends with raw Rust debug text ("HotKey already registered: HotKey { mods: Modifiers(ALT | CONTROL), key: KeyK, id: 589853 }"). Trim it to "Another app is probably using it."

Decide: default hotkey (recommended: `Ctrl+Alt+I`; `Ctrl+Alt+P` is the panel/strip toggle since the surface grill, AV-40). Decide: a separate centred overlay window vs focusing the docked widget (recommended: focus the docked widget; one surface).

## Decisions taken without a ruling

- Default hotkey `Ctrl+Alt+I`, as recommended. Rebindable in settings ("Inbox hotkey": press a combination, Enter or Save; Esc puts the live one back), stored as `inboxHotkey` in `%APPDATA%\PeepThis\settings.json` beside the trail switch. A combination must include Ctrl, Alt or Win; `Ctrl+Alt+P` (AV-40) and `Ctrl+Alt+Delete` are refused.
- One surface, as recommended: the hotkey raises and focuses the docked widget; the AV-23 cards under the rows are the inbox. No overlay window.
- Registered from Rust only (`src-tauri/src/hotkey.rs`, `tauri-plugin-global-shortcut` 2); no JS binding and no capability change. A clash at startup or on rebinding shows a persistent amber notice in the list with a Change button and the error in settings; a failed rebind keeps the old key. The toast hint on `shell`/`destructive` requests (AV-25) names the live key, or none when it is not registered.
- Hotkey with nothing pending (counted in Rust from the snapshot) flashes the header title plate in two stepped blinks and neither shows nor focuses the widget. Because the broker releases everything while hidden and holds nothing new, a hidden widget never has pending, so a press while hidden only flashes the hidden header. "Shows the widget if hidden" is still wired for when it does.
- The window to return to is remembered when the hotkey fires (never the widget itself; a second press from inside the widget keeps the first). `Esc`, and the inbox emptying while open, raise it again through the existing `focus.rs` path. `Enter` (terminal) and clicking away to another app forget it instead. A window that closed meanwhile is skipped.
- The broker needs no change: its foreground check compares the foreground window with the session's terminal, so the focused widget never counts. Returning focus to a terminal whose session still holds a request does release that request, as the spec's "terminal focused" rule says.
- Order is oldest first by `receivedAt` across all live sessions and subagents, ties by request id. After a decision the next request in the old order takes focus, wrapping to the oldest; `Tab`/`Shift+Tab` wrap.
- Keys act only while the inbox is open (after a hotkey press), so `Tab` and `Enter` behave normally otherwise. `R` opens the card's reason field; inside it `Enter` sends and the first `Esc` closes the field only. `Y` on a destructive card runs the card's own 1s hold (key down starts, key up or `Tab` cancels). The card shows a key hint line while it holds inbox focus.
- Pending count: a waiting-amber square badge on the title plate, `9+` above nine, tooltip naming the hotkey. It fits the 350px header without overflow (measured in headless Chrome).
- Not seen live: the hotkey firing, focus stealing past the Windows foreground lock (the press is user input, with the existing Alt-tap fallback), and focus returning to the starting window.
