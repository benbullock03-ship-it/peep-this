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
- [x] Say why the emit is lost (emitted before the listener exists after all, or dropped)
- [ ] Check every other event emitted during `setup` for the same loss: `sessions`, `mode`, `rules`, integration status, digest
- [ ] Fix at the source (order of `setup`, or commands that never answer with a placeholder), then remove `fetchHotkey`'s retry in `src/main.ts` if it is no longer needed
- [ ] A test or a recorded live check that fails before the fix

**Done when:** a cold launch of the release build shows correct hotkey, mode and session state with the retry removed, and the cause is written here.

## Cause (2026-11-15, read from source, not yet traced live)

The event is dropped by the page, because the page-side half of `listen` has not run yet. Sources: tauri 2.12.5, tauri-runtime-wry 2.12.3, wry 0.57.2, tao.

1. `setup` runs inside tao's `Ready` handler. `flyout::setup` builds a second webview, and wry waits for WebView2 with `wait_with_pump`, so the main page loads and runs its startup code while `setup` is still on the stack. The suspect in this ticket is right.
2. `plugin:event|listen` is an async command, so it runs off the main thread. `Webview::listen_js` does two things: it sends the script that registers the handler id in the page, and it records the listener in the backend. Sent from a worker thread, the script goes through the event-loop proxy, and tao buffers proxy events while a handler (`Ready`) is running. The backend record is immediate and the command answers, so `await listen(...)` resolves in the page with nothing registered there yet.
3. `hotkey::setup` then emits on the main thread. A main-thread eval skips the proxy and runs at once (`send_user_message`). The page's emit function finds no listener under those ids and returns without calling anything.
4. `setup` returns, the buffered registration scripts run, and the events are already gone.

So an emit is lost when it is made on the main thread, inside `setup`, after `flyout::setup`. Emits from worker threads queue behind the registrations and arrive.

Other events, same test:

- `hotkey-status`, `toggle-hotkey-status`: lost (the known case).
- `sessions`: `integrator.refresh` publishes on the main thread after `flyout::setup`, so that one push can be lost; the next push from a source thread or the ticker covers it. `get_snapshot` can also be asked before `app.manage(store)` and fail.
- `digest`: lost the same way if `away::wire` emits during `setup`; `get_digest` after `listen` covers it.
- `mode`, `rules`, `trail`, `integration-result`, `open-settings`, `widget-hidden`: not emitted during `setup`.

## Fix, written and reverted

Move `flyout::setup(app)?` to the last line of `setup` in `src-tauri/src/lib.rs`, with a one-line comment saying why. Nothing then emits on the main thread after the pump, and every command the page can reach already has its real state. Then delete `fetchHotkey` in `src/main.ts` and call `invoke` directly at its two call sites.

## Blocked 2026-11-15

Both files the fix changes (`src-tauri/src/lib.rs`, `src/main.ts`) hold the orphaned AV-33 hunks marked do-not-touch. Unblocks when the owner rules on AV-33 (adopt and commit it, or allow AV-47's hunks beside it).

## Owed before done

- The fix above, once unblocked
- A live trace or a cold launch of a release build with the retry removed. The installed widget was running, and the single-instance lock means a test build cannot start beside it
