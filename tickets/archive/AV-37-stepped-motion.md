---
id: AV-37
title: Stepped motion language
status: done
depends: [AV-35]
attended: false
---

# AV-37 — Stepped motion language

**What to build:** One motion rule for the whole widget: transitions run in 4 to 6 discrete frames over about 160ms, like the peeps' 8 to 12 fps (spec §12.3). This ticket sets the tokens and converts what exists; later surface tickets use the tokens.

- [x] CSS tokens for duration and `steps()` timing, defined once
- [x] Row enter, row leave, row flash and expand/collapse use the tokens
- [x] The waiting pulse stays a slow loop but steps between its levels
- [x] A Rust helper resizes or moves the window in the same number of steps, for AV-38, AV-39 and AV-41
- [x] `prefers-reduced-motion`: every transition is instant; the waiting row keeps its static amber outline
- [x] No smooth easing left in `styles.css` apart from colour hover fades under 150ms

**Done when:** rows entering, leaving and expanding in `?demo` visibly step, and with reduced motion on nothing animates.

## Decisions taken without a ruling

- 5 frames over 160ms (`--motion-ms`, `--motion-steps: steps(5, end)`), mirrored in `src/motion.ts` and `src-tauri/src/motion.rs`. `motion.test.ts` fails if the CSS and TS drift apart.
- Row leave dropped from 600ms to 160ms, a stepped fade with a 4px lift.
- Row flash is two stepped blinks over 480ms, which matches the header flash. Under reduced motion it shows a static tint for the same time, so a failed focus still gives feedback.
- The waiting pulse steps through four levels (0.12, 0.3, 0.5, 0.3) at its old 1.5s period.
- Expand reveals the stage and facts top-down with a stepped `clip-path`. Collapse reveals the closed row the same way.
- Still linear, on purpose: the 60s approval countdown bar and the 1s destructive hold fill. They show time passing rather than transitioning, so they also keep running under reduced motion.
- `window.rs` `apply_fit` stays a single instant resize. Stepping it would cut the card's border and shadow off for 4 frames while the window catches up with content that has already been laid out. `motion::animate` (one `SetWindowPos` per frame, with a newer call cancelling a running one) is ready for AV-38, AV-39 and AV-41 to adopt.
- `motion::reduced_motion` reads `SPI_GETCLIENTAREAANIMATION`, the same Windows switch WebView2 maps to `prefers-reduced-motion`.

## Done 2026-11-15

Tokens: `src/styles.css` `:root`. TS: `src/motion.ts`, `src/rows.ts` (`FLASH_MS`, enter and leave timers, the `closing` class). Rust: `src-tauri/src/motion.rs` (`frames`, `animate`, `reduced_motion`) and its 3 tests. `src/motion.test.ts` checks that the tokens match and that no `ease`/`cubic-bezier` is left in the CSS.

Checked by Claude in `?demo` with headless Edge, using `document.getAnimations()`:
- `row-open`, `row-leave` and `row-enter` run 160ms with `steps(5)` keyframes.
- `row-pulse` uses `steps(1)` across 4 levels; `row-flash` runs 480ms.
- With `prefers-reduced-motion: reduce` emulated, no row animation runs, only the countdown bar. The flash tint is static.
