---
id: AV-17
title: Windows installer build
status: todo
depends: [AV-08, AV-12]
attended: true
---

# AV-17 — Windows installer build

**What to build:** `npm run tauri build` producing an NSIS installer that includes the forwarder and statusline binaries (if AV-06/AV-07 needed them) at a stable path the installed hooks can point to. No publishing.

- [ ] Sidecar binaries bundled and resolved from the install directory
- [ ] Upgrade keeps settings and marks the integration `outdated` if binary paths changed
- [ ] Uninstaller offers to remove the Claude Code integration first

**Done when:** the owner has installed the built package and it works with live sessions. No release without an explicit yes.
