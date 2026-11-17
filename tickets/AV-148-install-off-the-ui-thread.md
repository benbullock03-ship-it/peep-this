---
id: AV-148
title: Run the integration install off the UI thread
status: open
depends: []
---

# AV-148: Run the integration install off the UI thread

**What to build:** the house Install button and the banner call `install_integration` (`src-tauri/src/install.rs`), a synchronous Tauri command, so the whole window freezes and Windows titles it "Peep This (Not Responding)" while the hooks merge and the `claude plugin` steps run (AV-129 live check, 2026-11-17). The tray already runs the same install on its own thread. Install is the newcomer's first click (9/10 bar), so a frozen window there reads as a crash.

- [ ] `install_integration` (and `uninstall_integration` if it is also synchronous) runs off the UI thread: an `async` command or `spawn_blocking`, returning the same `Result`
- [ ] The house bubble keeps "Installing…" while it runs and the window stays responsive
- [ ] A failure still reaches the bubble or banner as today
- [ ] A test that the command no longer blocks the caller, or the existing installer round trips still pass unchanged

**Done when:** in `npm run tauri dev` on a fresh profile, clicking Install leaves the window responsive (no "Not Responding" title) and the house moves on to "Start Claude Code in a project".
