---
id: AV-62
title: Integration install adds and removes the peep plugin
status: open
depends: [AV-55]
attended: true
---

# AV-62 — Install the pack with the widget

**What to build:** The widget's one-click integration install also installs the `peep` plugin, and uninstall removes it.

- [ ] Use Claude Code's own plugin install path, not hand-edited files, if one exists; otherwise the same backup-then-merge rule as hooks
- [ ] Uninstall reverses it; the owner's own skills and plugins are untouched
- [ ] Installer round-trip test in `cargo test`

**Done when:** install then uninstall on a clean profile leaves `~/.claude` as it was, and the owner has confirmed `/peep:` commands appear after install.
