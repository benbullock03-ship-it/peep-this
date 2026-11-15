---
id: AV-66
title: Inserted floppy on the plank and ticket-aware bubble
status: open
depends: [AV-63, AV-64]
attended: true
---

# AV-66 — Inserted floppy and bubble

**What to build:** A row shows the running skill and the ticket, per the AV-63 design.

- [ ] Inserted floppy on the plank while a skill runs; eject frames on Stop (stepped motion, instant under reduced motion)
- [ ] Bubble shows ticket and skill ("AV-34: chip contrast · shooting") per the design's truncation rules
- [ ] Attention states (`waiting`, `error`, `stuck`) take over the bubble as today
- [ ] Vitest for the bubble text builder (with and without ticket, truncation, attention takeover)
- [ ] Shoot both themes and both widths

**Done when:** the owner has seen a live `/peep:ticket` run insert and eject the floppy in `npm run tauri dev`.
