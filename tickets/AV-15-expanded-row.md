---
id: AV-15
title: Expanded row details
status: in-progress
depends: [AV-04, AV-07, AV-09]
attended: false
---

# AV-15 — Expanded row details

**What to build:** Click-to-expand per spec §3.4: 64px peep, cwd, model, last tool with target, time in state (live), context used %, kind and entrypoint. One row open at a time.

- [x] Peep resizes 32 → 64 without restarting its animation
- [x] Time in state ticks every second while expanded only
- [x] Long cwd and tool targets truncate from the middle

**Done when:** the owner has expanded rows for live sessions and the details matched.

## Decisions taken without a ruling

- Started with AV-04, AV-07 and AV-09 `in-progress`: all three are built and owe only the owner's live check, as AV-14 did.
- Clicking anywhere on a row except the peep and name toggles it; peep and name still focus the terminal (§3.6). Enter/Space on a focused row also toggles. In `?demo` (no focus handler) the whole row toggles.
- Context shows `NN% used`: the snapshot carries only `contextPct`, not a max token count, and the spec rules out token counts.
- The backend's 60-char target cap (`state.rs` `truncate`) now cuts from the middle too, so a long path keeps its filename before the frontend's 38-char middle cut.
- No `design-tropes` skill is installed; checked with Playwright shots of `?demo` at 360px instead.

## Owed before done

Built 2026-11-13. `src/details.ts` (fields, `truncateMiddle`, `formatDuration`; `src/details.test.ts`), expand/collapse and the 1s ticker in `src/rows.ts` (one interval, only while a row is open), styles in `src/styles.css`, demo rows carry model, a long cwd and a long tool target. The peep keeps its element and only its `size` attribute changes, so peep.js's animation frame carries on. Owed: the owner expands rows for live sessions in `npm run tauri dev` and confirms the details match.
