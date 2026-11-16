---
id: AV-111
title: Focus-aware toasts and an optional alert sound
status: closed
depends: []
---

# AV-111 — Focus-aware toasts and an optional alert sound

**What to build:** Two alert fixes competitors' users ask for most.

- [x] No toast when the session's own terminal window (the one `focus_session` would raise) is already foreground; the row still pulses
- [x] Optional sound on entering `waiting` or `error`, off by default, toggled in settings; never for subagent completion or `success`
- [x] Rust test for the foreground check against a fake window resolver; update spec §7

**Done when:** with the session's terminal focused no toast appears, with it unfocused one does, and the sound plays only when switched on; the owner confirms.

## Owed before done

- Unfocused case passed live 2026-11-16: three waiting sessions with terminals in the background each raised a toast (AV-112 run).
- Owner check in `npm run tauri dev`: trigger a waiting with that session's terminal focused (no toast, row pulses); switch "Play a sound on waiting or error" on and hear the Asterisk sound on `waiting` and on an API error, and nothing on `success` or a finished subagent.
- Proven so far: `notify::tests::notify_no_toast_while_the_sessions_terminal_is_foreground`, `notify_sound_only_when_on_and_only_for_waiting_and_error`, `notify_alert_entry_is_waiting_or_error_never_success`, `focus::tests::focus_foreground_only_when_the_resolved_window_is_in_front`; settings toggle shot in `?demo` (light/dark, 375/1280).

## Decisions taken without a ruling

- "Spec §7" read as §3.6 Alerts (§7 is Integration install); the toast and sound rules went there.
- The waiting toast is now silent: its built-in default sound would otherwise play with the switch off. Rate-limit toasts keep their default sound. Owner kept this 2026-11-16.
- Sound is `MessageBeep(MB_ICONASTERISK)` (Windows Asterisk system sound, needs the `Win32_System_Diagnostics_Debug` windows-sys feature); one sound for both `waiting` and `error`, no bundled file.
- The sound plays on every entry into `waiting` or `error`, even when the toast is suppressed (terminal or widget focused, 30s de-dup); no de-dup of its own.
- A toast suppressed because the terminal is foreground does not start the 30s de-dup window, matching the existing widget-focused rule.
- Setting stored as `alertSound` in the widget's `settings.json`, alongside the other widget settings.
