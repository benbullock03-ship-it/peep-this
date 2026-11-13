---
id: AV-18
title: Rename AgentView to Peep This
status: done
depends: [AV-05, AV-09, AV-11, AV-13]
attended: false
---

# AV-18 — Rename AgentView to Peep This

**What to build:** the product is "Peep This" (owner, 2026-11-13; full rename chosen). Rename everywhere before tray, autostart and installer bake the old name in.

| Old | New |
|---|---|
| `AgentView` (display, productName, window title, header, notices) | `Peep This` |
| identifier `com.agentview.widget` | `com.peepthis.widget` |
| `%APPDATA%\AgentView\` | `%APPDATA%\PeepThis\` |
| crate `agentview` / lib `agentview_lib`, npm `agentview` | `peep-this` / `peep_this_lib`, `peep-this` |
| log prefix `agentview:` | `peepthis:` |
| header `X-AgentView-Token` | `X-PeepThis-Token` |
| `agentview-hook.exe`, `agentview-statusline.exe` | `peepthis-hook.exe`, `peepthis-statusline.exe` |
| `settings.json.agentview-backup-<ts>` | `settings.json.peepthis-backup-<ts>` |
| temp dirs `agentview-test-*` | `peepthis-test-*` |

- [x] Code, config, CLAUDE.md, spec, open tickets (AV-06..AV-17) and in-progress tickets updated
- [x] Test fixture paths that are real folder names (`F:\Projects\agentview widget`, `...\Projects\AgentView`) are left alone: they are data, and identity hashes depend on them
- [x] The repo folder and `~/.claude/projects` memory slug are not renamed

**Done when:** no `AgentView`/`agentview` remains outside fixture path data, closed-ticket history and `Cargo.lock`/`package-lock.json` regenerated; full run passes.

## Decisions taken without a ruling

- Env var `AGENTVIEW_FOCUS_PID` (ignored live focus test) renamed to `PEEPTHIS_FOCUS_PID`; temp dirs `agentview-ingest-*` and `agentview-transcript-test-*` follow the `peepthis-` prefix too.
- Kept as data: `focus.rs` window-title/folder-hint tests and `transcript.rs` slug test (they use the real repo folder name), AV-09's slug evidence, plus the fixtures the ticket lists.
- Existing `%APPDATA%\AgentView\window.json` and `endpoint.json` are not migrated: the window falls back to the default dock once, and the endpoint is rewritten on every start.
