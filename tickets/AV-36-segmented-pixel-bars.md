---
id: AV-36
title: Segmented pixel bars for context and rate limits
status: todo
depends: []
attended: false
---

# AV-36 — Segmented pixel bars for context and rate limits

**What to build:** The context bar under each row and the 5h/7d bars in the header change from thin smooth lines to rows of square blocks that fill one at a time (spec §12.2).

- [ ] One shared bar component used by rows and the header; 10 blocks, square, on the 2px grid
- [ ] Filled blocks = used % rounded to the nearest block; any value above 0 lights at least one
- [ ] Filled blocks are neutral below 70%, amber from 70%, red from 90%
- [ ] The percentage text in the header stays; rows still show no number in compact form
- [ ] Bar is absent, not empty, until a value exists (as today)
- [ ] A block changes state in one step, no width tween
- [ ] Vitest for the block count and colour thresholds

**Done when:** `?demo` shows rows at low, 70%+ and 90%+ context with the right block counts and colours, and the owner confirmed it in `npm run tauri dev`.

Decide: thresholds 70/90 (recommended) vs matching Claude Code's own context warnings if those differ.
