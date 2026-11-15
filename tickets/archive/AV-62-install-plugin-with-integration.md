---
id: AV-62
title: Integration install adds and removes the peep plugin
status: closed
depends: [AV-55]
attended: true
---

# AV-62 — Install the pack with the widget

**What to build:** The widget's one-click integration install also installs the `peep` plugin, and uninstall removes it.

- [ ] Use Claude Code's own plugin install path, not hand-edited files, if one exists; otherwise the same backup-then-merge rule as hooks
- [ ] Uninstall reverses it; the owner's own skills and plugins are untouched
- [ ] Installer round-trip test in `cargo test`

**Done when:** install then uninstall on a clean profile leaves `~/.claude` as it was, and the owner has confirmed `/peep:` commands appear after install.

## Notes (2026-11-15)

- `install.rs`: after the hooks are written, install runs `claude plugin marketplace add <plugin_dir>` then `claude plugin install peep@peep-this`; uninstall runs `claude plugin uninstall peep@peep-this`, `claude plugin marketplace remove peep-this`, then deletes `<claude dir>/plugins/cache/peep-this` (that exact folder only, and only after the marketplace remove succeeded).
- Seam: `ClaudeCli` (`dyn Fn(&[&str]) -> io::Result<()>`), real impl `run_claude` (tries `claude`, then `claude.cmd`; no console window). Tests pass a recording fake or `no_claude`.
- Plugin step never fails the install: returns `PluginStep::Skipped(reason)` when `claude` is not on PATH, the plugin dir is missing or the CLI errors. Hooks stay installed.
- `tauri.conf.json`: `bundle.resources` `{"../plugin/": "plugin/"}`; release reads `<exe dir>/plugin`, debug builds read the repo `plugin/`.
- Tests: `plugin_round_trip_uses_the_claude_cli_and_restores_settings`, `plugin_step_is_skipped_without_claude_or_plugin_dir`; `cargo test install` 18 passed.
- Owed: the owner runs the one-click install and confirms `/peep:hello` appears in a fresh session, then uninstall leaves `~/.claude/plugins` as before.

## Decisions taken without a ruling

- Plugin is installed through the `claude plugin` CLI (Claude Code's own path), not by editing `installed_plugins.json`/`known_marketplaces.json`.
- A failing `marketplace add` is ignored unless `claude` is missing; the `install` call decides the outcome (covers re-install over an existing marketplace).
- Uninstall removes the `peep-this` marketplace even if the owner added it by hand; the name is ours.
- The skip reason is only logged (stdout); the Tauri commands still return `()`, so the UI does not show it yet.
- `integration_status` does not check the plugin; Installed/Outdated still mean hooks and statusline.
- Uninstall with no `settings.json` still runs the plugin removal.
