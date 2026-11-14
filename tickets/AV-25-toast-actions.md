---
id: AV-25
title: Allow and Deny buttons on the waiting toast
status: in-progress
depends: [AV-21, AV-22]
attended: false
---

# AV-25 — Allow and Deny buttons on the waiting toast

**What to build:** The waiting toast (spec §3.6) gains action buttons for low-risk requests (spec §11.3), so a `safe`-tier approval is one click from wherever the user is.

- [x] `safe` tier: toast shows tool and target plus Allow and Deny. `shell` and `destructive`: no buttons; body says to open the inbox (hotkey shown)
- [x] Buttons route to `decide_approval`; clicking the toast body still focuses the terminal
- [x] A toast whose request already resolved does nothing when clicked, and is removed from the Action Center if the API allows
- [x] De-duplication rules from AV-14 unchanged

**Done when:** the owner allowed a file read from a toast while another app was focused, and saw that a shell command's toast offered no buttons.

Decide: does `tauri-winrt-notification` deliver button activations in-process without a COM activator? If not, recommended: drop this ticket and rely on the hotkey inbox.

## Built 2026-11-14

- `src-tauri/src/notify.rs`: `toast_content(session)` picks the newest held request. `safe` gets Allow and Deny buttons (arguments `allow:<id>`, `deny:<id>`); `shell` and `destructive` get no buttons and a second line "Review it in Peep This (Ctrl+Alt+I)" (`INBOX_HOTKEY`). A wait with no held request (Notification only) toasts as before. `click_action` routes a button to a decision only while that request is still pending; a body click focuses the terminal unless the toast's request has resolved, then does nothing. `toast()` now takes the content and passes the button argument through.
- `src-tauri/src/lib.rs`: the alert builds the toast on its own thread `SHOW_DELAY_MS` (200ms) later from the current session, because the hook's state event enters `waiting` (and fires the alert) just before the broker records the held request. Buttons call `Broker::decide` directly, the same path as `decide_approval`; Deny sends the default reason.
- De-duplication (`Notifier`, `should_toast`, `DEDUP_MS`) untouched.
- Tests: `cargo test notify` (4 new: safe buttons, shell/destructive hint, newest request and plain waits, click routing including resolved toasts and bad arguments).

## Decisions taken without a ruling

- In-process activation: the crate's `add_button` sets the button `arguments`, and the existing `on_activated` handler (which already carries AV-14 body clicks without a COM activator) receives them as `ToastActivatedEventArgs.Arguments`. So the ticket stays; the owner's live check is the proof. If buttons do nothing live, fall back to the recommendation and drop the buttons.
- Not removed from the Action Center: the crate exposes no tag or group setter (only via a progress bar), and `ToastNotificationHistory::Remove` needs one. A stale toast is inert instead. Doing it properly means building the toast XML with the `windows` crate directly; not worth it for this ticket.
- Hotkey text is `Ctrl+Alt+I`, AV-24's recommended default, ahead of AV-24 registering it.
- Toast Deny uses the broker's default reason ("Denied from Peep This."). Deny with reason stays on the card.
- The 200ms delay re-reads the session; if the request resolved in that window the toast shows without buttons.

## Owed before done

- Owner, with the widget unfocused and another app in front: a file read (or in-project edit) toast shows Allow and Deny, and Allow goes through; a shell command's toast shows no buttons and the inbox line. If the buttons appear but do nothing, the in-process route fails and the buttons come out.
- Checks: `npm run build` clean, 56 Vitest, 233 + 15 cargo tests.
