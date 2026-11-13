---
id: AV-08
title: One-click integration install and uninstall
status: in-progress
depends: [AV-06, AV-07]
attended: true
---

# AV-08 — One-click integration install and uninstall

**What to build:** `install.rs` per spec §7, plus tray menu items and a first-run banner offering install. This edits the owner's real `~/.claude/settings.json`, hence attended.

- [x] Timestamped backup before any write
- [x] Merge keeps every existing hook; Peep This entries identified by their loopback `/hook/` URL (native http hooks since AV-06, so no command path); reinstall idempotent
- [x] Statusline original recorded and wrapped; uninstall restores it exactly
- [x] Temp file + atomic rename; result must parse before replacing
- [x] `integration` status `installed` / `missing` / `outdated` in the snapshot
- [x] Round-trip tests on fixture settings files (with and without existing hooks/statusline): install → uninstall yields the original bytes' JSON

**Done when:** tests pass, and the owner has run install and uninstall on their machine and confirmed settings are intact.

## Unblocked 2026-11-13

AV-07's wrapper and `statusline_original.rs` landed in a714f49. AV-07 and AV-04 still owe only the owner's live check, which does not gate building this.

## Built 2026-11-13

`src-tauri/src/install.rs` (install, uninstall, `integration_status`, commands `install_integration` / `uninstall_integration`), `Store::set_integration`, status computed at startup and after each command. Frontend: `src/banner.ts` above the rows. Fixtures in `src-tauri/tests/fixtures/settings/`. 15 install tests plus one store test.

## Decisions taken without a ruling

1. **Key order:** serde_json `preserve_order` is on, so settings keep their key order; keys are removed with `shift_remove`. Output is pretty-printed, 2-space indent, trailing newline, LF, no BOM (a BOM on input is tolerated). A CRLF settings file comes back LF: same JSON, different bytes.
2. **Backup:** `settings.json.peepthis-backup-<YYYYMMDD-HHMMSS>` in UTC, written from the exact bytes read, before any write, on both install and uninstall. A second backup in the same second gets `-2`, `-3`, so the pre-install copy is never overwritten. No settings.json: install creates it from `{}` with no backup, and uninstall then leaves `{}`.
3. **Wrapper location:** install copies `peepthis-statusline.exe` from next to the running exe to `%APPDATA%\PeepThis\bin\` (skipped when the bytes already match; a running copy is renamed aside, then replaced) and writes `statusLine.command` as that path with forward slashes, double-quoted if it contains a space. Other keys of the existing `statusLine` (e.g. `padding`) are kept. A missing source exe fails the install before anything is written.
4. **Ours = loopback http hook:** `type: "http"` and a URL starting `http://127.0.0.1:` containing `/hook/`. Install first removes ours from every event (also events no longer installed, e.g. an old `SessionStart`), dropping groups and event arrays that removal emptied, then appends one matcher group per event. Install twice is byte-equal to install once.
5. **Statusline record:** a `statusLine` whose command contains `peepthis-statusline` is ours and is never recorded as the original. Absence is recorded as `{"statusLine": null}`; uninstall then removes the key.
6. **Uninstall pruning:** only groups, event arrays and the `hooks` object that held our entries are removed when emptied. Edge case: a user's pre-existing empty `"hooks": {}` or `"Stop": []` that we added into is removed on uninstall, because nothing records that it existed empty. If the user changed `statusLine` since install it is left alone (logged). If the record is missing while `statusLine` is still ours, the key is removed. `statusline-original.json` is deleted after uninstall; the copied exe is left (harmless).
7. **Atomic write:** serialise, re-parse, write `settings.json.peepthis-tmp`, rename over. Invalid JSON or a non-object root refuses both install and uninstall and writes nothing.
8. **Status:** `installed` needs every event to carry exactly our entry with the current port and token, `statusLine` pointing at the expected exe path, and that exe byte-equal to the one next to the running app. `outdated` when any of ours exist but something differs (also when the source exe is missing, e.g. a dev build without the wrapper). `missing` when none exist or settings.json is unreadable/invalid.
9. **Fallback port:** `install_integration` refuses when the server is not on 47821, because hooks hold the port literally and would break on the next restart.
10. **Banner:** shown for `missing` (Install) and `outdated` (Update). × remembers the dismissed status in localStorage, so a later change (missing → outdated) shows it again. Errors show inline. Not shown in `?demo`.
11. **Uninstall UI stopgap:** right-click the header to open the banner on demand: Disconnect when installed, Install/Update otherwise (also reopens a dismissed banner). After a disconnect the install banner stays dismissed. AV-12 owes the tray items and removes this (line added there).
12. Settings path honours `CLAUDE_CONFIG_DIR`, like the sessions source.

## Notes for AV-17

The release bundle must ship `peepthis-statusline.exe` next to the main exe (Tauri `externalBin` sidecar lands there with the triple suffix stripped); install looks for it at `current_exe().parent()/peepthis-statusline.exe`. An upgrade that changes the wrapper's bytes reads as `outdated` until Update is clicked.

## Owed before done

The owner runs this on their machine (attended):

1. Build the wrapper next to the dev exe (`tauri dev` builds only the main binary): `cd src-tauri; cargo build --bin peepthis-statusline; cd ..`
2. `npm run tauri dev`. The banner "Connect Peep This to Claude Code" shows; click Install. It disappears; the console logs the backup path.
3. Confirm: `~/.claude/settings.json` keeps every original key and hook, has one `127.0.0.1:47821/hook/<Event>` group per event, `statusLine.command` is `C:/Users/<you>/AppData/Roaming/PeepThis/bin/peepthis-statusline.exe`, and `%APPDATA%\PeepThis\statusline-original.json` holds the old `sh ~/.claude/statusline-command.sh` entry. A new Claude Code session shows the same statusline as before and its row gets hook states.
4. Uninstall: tray menu → Uninstall integration (AV-12 replaced the header right-click stopgap). (Alternative: devtools console, `__TAURI__.core.invoke("uninstall_integration")`; `withGlobalTauri` is on.)
5. Diff against the first backup, e.g. `git diff --no-index ~/.claude/settings.json.peepthis-backup-<stamp> ~/.claude/settings.json`: only whitespace/line-ending or formatting differences are expected, no content changes.
