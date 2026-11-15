---
id: AV-47
title: Startup status event never reaches the page
status: todo
depends: []
attended: false
---

# AV-47 — Startup status event never reaches the page

**What to build:** Find out why an event the backend emits during `setup` does not reach a listener the page has already registered, and fix it at the source.

Found 2026-11-15 on the installed release build (3 launches of 3). The page asked `inbox_hotkey` / `toggle_hotkey` before `hotkey::setup` had run and got the placeholder status (not registered, no error). `hotkey::setup` then registered both keys and emitted `hotkey-status` / `toggle-hotkey-status`; neither listener fired, so both "not active" notices stayed up while the hotkeys worked. Commit `074c54d` works around it for the hotkeys only: the page re-asks until it gets a real answer.

- [ ] Reproduce with a trace: when the page's `listen` and first `invoke` land relative to each step of `setup` in `lib.rs`. Suspect: `flyout::setup` creating the second webview pumps messages, so the main page runs during `setup`
- [ ] Say why the emit is lost (emitted before the listener exists after all, or dropped)
- [ ] Check every other event emitted during `setup` for the same loss: `sessions`, `mode`, `rules`, integration status, digest
- [ ] Fix at the source (order of `setup`, or commands that never answer with a placeholder), then remove `fetchHotkey`'s retry in `src/main.ts` if it is no longer needed
- [ ] A test or a recorded live check that fails before the fix

**Done when:** a cold launch of the release build shows correct hotkey, mode and session state with the retry removed, and the cause is written here.
