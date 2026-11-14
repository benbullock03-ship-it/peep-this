---
id: AV-36
title: Segmented pixel bars for context and rate limits
status: in-progress
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

## Decisions taken without a ruling

- Thresholds 70/90 (the recommendation), on the raw used %, shared by context and both rate limits (`src/bar.ts`).
- Blocks are 4px squares with 2px gaps (58px per bar), the same size in rows and the header. Empty blocks use the track colour, filled ones `--text-3`, amber `rgb(--amber)`, red a new `--red` token (`#ff5d5d`, the existing error red).
- The row bar no longer spans the row; it sits at the row's bottom under the text column (left 56px, 88px when expanded), since square blocks cannot stretch to the full width.
- `?demo` context values are now 74 (amber), 35, 93 (red), 12, 47 and none; demo rate limits stay 42/18 (a test pins them).

## Owed before done

- Owner check in `npm run tauri dev` with `?demo`: ingest-01 7 amber, triage-03 9 red, research-02 / digest-04 / sync-05 4 / 1 / 5 neutral, scrape-06 no bar; row bar sits under the name and follows expand; header 5h/7d show % over blocks and still drags; blocks switch in one step.
