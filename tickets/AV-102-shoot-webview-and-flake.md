---
id: AV-102
title: Shoot - desktop webviews over CDP and rules against flaky shots
status: done
depends: []
---

# AV-102 — Shoot: desktop webviews and flake rules

**What to build:** `/peep:shoot` covers a desktop app's webview (Tauri, WebView2, Electron) and runs checks that give the same verdict twice. Techniques come from the AV-96/AV-91 live checks on 2026-11-16, where Claude ran the owner's checks itself.

- [x] `references/webview.md`: launch with a remote-debugging port, attach over CDP for DOM facts, invoke the app's own commands, single-instance and inherited-env traps, OS screenshot for native chrome, app logs as evidence, clean up what was started
- [x] `references/stable.md`: rules that keep verdicts repeatable (wait on conditions, fonts, motion, clocks and live data, pointer and focus leftovers, fixed viewport, one retry then "flaky")
- [x] SKILL.md: setup accepts a `webview` target; DOM and shot steps cite `stable.md`; housekeeping covers processes; "native apps" line narrowed to native-only UI
- [x] Never drive a window blind with keystrokes; use the app's commands or ask the owner

**Done when:** both references exist, SKILL.md cites them, and the rules match what worked on 2026-11-16.

## Done 2026-11-16

- `plugin/skills/shoot/references/webview.md` and `stable.md`; SKILL.md cites them in §1, §4 and §7, adds a `flaky` verdict, and narrows the "Not here" line.
- Each webview rule is one used in the AV-96 check (CDP `Runtime.evaluate` + `invoke`, single-instance handoff, reused vite, cleared `CLAUDE*` env, OS screenshot) or a failure hit there (keystrokes into an unseen trust prompt).

## Decisions taken without a ruling

- Changed the plugin's `peep:shoot` only; the owner's global `~/.claude/skills/shoot` is untouched.
- CDP by a scratch script, not the Playwright MCP: the MCP takes a CDP endpoint only at server start.
- `flaky` added as a fourth verdict rather than folded into `can't tell`, so a moving result is never read as missing evidence.
