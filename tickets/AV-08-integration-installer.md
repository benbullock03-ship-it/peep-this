---
id: AV-08
title: One-click integration install and uninstall
status: todo
depends: [AV-06, AV-07]
attended: true
---

# AV-08 — One-click integration install and uninstall

**What to build:** `install.rs` per spec §7, plus tray menu items and a first-run banner offering install. This edits the owner's real `~/.claude/settings.json`, hence attended.

- [ ] Timestamped backup before any write
- [ ] Merge keeps every existing hook; AgentView entries identified by command path; reinstall idempotent
- [ ] Statusline original recorded and wrapped; uninstall restores it exactly
- [ ] Temp file + atomic rename; result must parse before replacing
- [ ] `integration` status `installed` / `missing` / `outdated` in the snapshot
- [ ] Round-trip tests on fixture settings files (with and without existing hooks/statusline): install → uninstall yields the original bytes' JSON

**Done when:** tests pass, and the owner has run install and uninstall on their machine and confirmed settings are intact.
