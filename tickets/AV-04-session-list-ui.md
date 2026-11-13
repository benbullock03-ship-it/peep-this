---
id: AV-04
title: Compact session rows with peeps
status: todo
depends: [AV-03]
attended: false
---

# AV-04 — Compact session rows with peeps

**What to build:** The frontend list from spec §3.3 and §3.5, fed by the `sessions` event: 32px peep, mono name, task text, LED + label, context bar slot, subagent badge. Waiting-first sort.

- [ ] Identity hash (FNV-1a over normalised cwd) to colour + accessory, in its own module with Vitest stability tests
- [ ] Sort tests: `waiting` first, then most recent `stateSince`
- [ ] LED colours and labels taken from the design (`design/peep-this.dc.html` STATE_META)
- [ ] Rows update in place (no re-creating `<peep-bot>` elements, so animations do not restart)
- [ ] Empty state: "No Claude Code sessions running"
- [ ] `?demo` query flag renders the six mock agents from the design for UI work without live sessions

**Done when:** `npm test` passes and live sessions appear as rows in `npm run tauri dev`; the owner has seen it.
