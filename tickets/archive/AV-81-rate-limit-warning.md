---
id: AV-81
title: Rate-limit warning in header, bubble, toast and title
status: closed
depends: [AV-69, AV-70, AV-79]
attended: true
---

# AV-81 — Rate-limit warning

**What to build:** Usage is readable at a glance and the peep warns before the limit hits, per the AV-79 design.

- [x] Header shows the % beside the 5h and 7d meters (today hover-only, `src/limits.ts`)
- [x] At 80% of either limit, every peep shows the sweat emote with bubble text naming the limit and % (mood in the AV-69 engine; attention states still win)
- [x] Windows toast once at 95% and once at 100% per limit window, with the reset time if the statusline payload carries one
- [x] The "Peep This" title reacts at 80 / 95 / 100% per the design; instant under reduced motion
- [x] Vitest for thresholds and the once-per-window toast rule; Shoot both themes

**Done when:** the owner has seen the header numbers, the 80% bubble and the 95% toast with a faked statusline payload in `npm run tauri dev`.

## Decisions taken without a ruling

- Meters with numbers don't fit beside the title and two 28px buttons in the 354px card (needs ~280px of 240px), so the meters take a second header line under the title (`src/styles.css` `.ps-head::after`).
- Limit sweat ranks after startled, cheer and greet (brief moods) and before worry, yawn and nap, so the context nudge waits while a limit is at 80% or more.
- The bubble names the higher limit; 5h on a tie. Limit text shows only while the row's mood is sweat, so a clash bubble or a brief mood keeps its own text.
- A limit window is its `resets_at`; a later one re-arms both toasts, an earlier one (stale payload) is ignored. With no `resets_at`, a drop under 80% re-arms. A jump straight past 95 shows only the 100% toast.
- `resets_at` is read as epoch seconds, epoch ms or ISO 8601; it rides in `RateLimits` (`fiveHourResetsAt`/`sevenDayResetsAt`) and the header tooltip shows it.
- Reset clock is local `HH:MM`, prefixed with a weekday when not today; a reset in the past reads "Reset time not known yet."; 95% with no reset reads "Reset time not known yet. Anything you start now may stop partway."
- Limit toasts show whatever the widget's view (not suppressed when focused); a click opens the panel (`window::summon`). They carry the app icon only: no house-peep image file exists to attach.
- The once-per-window toast rule lives in Rust, so its tests are `cargo test limit` rather than Vitest; Vitest covers the thresholds and bubble text.
- `?demo&limits=82,30` sets demo limits (with the sweat `mood.rs` would give) for Shoot.

## Done 2026-11-16

- Live in `npm run tauri dev` with faked `/statusline` payloads over the real loopback port: at 82 the header read "5h 82%" in amber, the title plate took the amber inset, and every peep had `mood: sweat` with "5h limit 82%" in the bubble. At 96 the plate filled amber and the meters went red; 97 sent no second toast; at 100 the title read "PEEP THIS · FULL" on the red hatch.
- Windows notification database holds "5-hour limit at 95%" / "Resets at 13:10…" and "5-hour limit reached" / "…in 2h 59m", once each.
- Meters on a second header line stay (owner closed it as built).
