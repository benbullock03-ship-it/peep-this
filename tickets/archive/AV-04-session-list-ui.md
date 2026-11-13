---
id: AV-04
title: Compact session rows with peeps
status: done
depends: [AV-03]
attended: false
---

# AV-04 — Compact session rows with peeps

**What to build:** The frontend list from spec §3.3 and §3.5, fed by the `sessions` event: 32px peep, mono name, task text, LED + label, context bar slot, subagent badge. Waiting-first sort.

- [x] Identity hash (FNV-1a over normalised cwd) to colour + accessory, in its own module with Vitest stability tests
- [x] Sort tests: `waiting` first, then most recent `stateSince`
- [x] LED colours and labels taken from the design (`design/peep-this.dc.html` STATE_META)
- [x] Rows update in place (no re-creating `<peep-bot>` elements, so animations do not restart)
- [x] Empty state: "No Claude Code sessions running"
- [x] `?demo` query flag renders the six mock agents from the design for UI work without live sessions

**Done when:** `npm test` passes and live sessions appear as rows in `npm run tauri dev`; the owner has seen it.

## Done 2026-11-13

Owner ran the installed build: live sessions show as rows (screenshot of `agentview-widget-24` in `waiting`). The check that was owed:

Owner runs `npm run tauri dev` (and `npm run dev` then `/?demo`) and confirms: live rows appear, peep animations don't restart on state change/reorder, 3-column layout reads at widget width.

## Decisions taken without a ruling

- Identity bits: colour = `h % 5`, accessory = `(h >>> 16) % 7` over 32-bit FNV-1a of the UTF-8 normalised cwd; outputs pinned in `src/identity.test.ts`, so changing this re-skins every project.
- Normalisation also trims surrounding whitespace and collapses repeated trailing slashes.
- Row layout scaled to ~360px: 3 columns (32px peep | name over task | LED + label) instead of the mock's 4 side-by-side columns, which do not fit.
- Context bar is a 2px track along the row's bottom edge, neutral grey fill; hidden when `contextPct` is absent. Colour thresholds left to AV-07/AV-16.
- Subagent badge is a small accent-yellow pill on the peep's top-right corner.
- Sort ties (same `stateSince`) break by session id so rows do not jitter.
- Empty `task` falls back to the state label; empty `name` falls back to the first 8 chars of the session id.
- `?demo` uses the design's fixed colours/accessories (not cwd hashes) and runs the design's simulation (one agent changes state every 3.2s) so in-place updates can be eyeballed.
- Ended sessions (`alive: false`) are dimmed via `data-alive`; the fade/removal itself stays with AV-10.
- Plain browser without `?demo` shows the empty state plus a one-line notice pointing at `?demo`.
- Frontend subscribes to `sessions` before calling `get_snapshot`; if an event lands first, the fetched snapshot is ignored.
