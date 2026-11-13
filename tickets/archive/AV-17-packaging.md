---
id: AV-17
title: Windows installer build
status: done
depends: [AV-08, AV-12]
attended: true
---

# AV-17 — Windows installer build

**What to build:** `npm run tauri build` producing an NSIS installer that includes the forwarder and statusline binaries (if AV-06/AV-07 needed them) at a stable path the installed hooks can point to. No publishing.

- [x] Sidecar binaries bundled and resolved from the install directory
- [x] Upgrade keeps settings and marks the integration `outdated` if binary paths changed
- [x] Uninstaller offers to remove the Claude Code integration first

**Done when:** the owner has installed the built package and it works with live sessions. No release without an explicit yes.

## Decisions taken without a ruling

- Started with AV-08 and AV-12 `in-progress`: both are built and owe only the owner's live check.
- No forwarder binary: hooks POST straight to the loopback port (AV-06), so the only sidecar is `peepthis-statusline.exe`.
- The wrapper ships through NSIS hooks (`src-tauri/windows/installer.nsh`), not `bundle.externalBin` or `resources`. Both of those make `tauri-build` require the file at `cargo build` time, which would break `cargo test` on a clean checkout. `beforeBuildCommand` builds the wrapper bin first; the hook finds it next to the main binary.
- The stable path stays `%APPDATA%\PeepThis\bin\peepthis-statusline.exe` (AV-08). The app copies it there from `$INSTDIR`, so moving the install directory never breaks the statusLine. An upgrade that changes the wrapper's bytes shows `outdated` (existing `status_outdated_cases` test) and the banner offers to re-run install.
- Uninstall asks Yes/No before unhooking. Silent uninstalls answer No (never edit the user's settings without asking). Upgrades (`/UPDATE`) never ask. If unhooking fails, OK uninstalls anyway and Cancel aborts so the tray menu can be used.
- `%APPDATA%\PeepThis` is deleted only when "delete app data" is ticked *and* the integration was removed. A kept statusLine still runs the wrapper copy in there.
- Target narrowed to NSIS only, per-user install (no UAC prompt).

## Done 2026-11-13

Owner installed the package and it works with live sessions (screenshot). They also installed a newer build over the old one with settings kept, and uninstalled through the Yes/No prompt with `settings.json` restored. Nothing was published.

The check that was owed:

Built 2026-11-13. `npm run tauri build` produced `src-tauri/target/release/bundle/nsis/Peep This_0.1.0_x64-setup.exe` (1.88 MiB). makensis compiled the hook's `File` line for `peepthis-statusline.exe` (it fails on a missing source). Headless `--uninstall-integration` flag in `main.rs` calls `install::remove_for_app_uninstall` (test `app_uninstall_restores_settings_and_removes_the_wrapper_copy`). Not installed on this machine by the agent.

Owner, in order:
1. Run the setup exe. The widget starts and the tray works. `%LOCALAPPDATA%\Peep This\` holds both exes.
2. Install integration from the banner or tray. statusLine points at `%APPDATA%/PeepThis/bin/peepthis-statusline.exe`, and live sessions show context and state.
3. Bump `version`, rebuild and run the new setup over the old one. Settings and integration stay; the banner shows `outdated` only if the wrapper changed.
4. Uninstall from Apps. The Yes/No prompt appears, and Yes restores `settings.json` (backup alongside).