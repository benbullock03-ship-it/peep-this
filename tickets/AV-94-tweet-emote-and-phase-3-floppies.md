---
id: AV-94
title: Tweet emote, phase 3 floppies and graph page style
status: done
depends: [AV-70, AV-90, AV-91, AV-92, AV-93]
attended: true
---

# AV-94 — Phase 3 art in the widget and graph page

**What to build:** Apply the AV-93 deliverables.

- [x] Add `bigpeep`, `tweet`, `goose` to the floppy list (`src/tray.ts`) and the backend skill mapping
- [x] Tweet mood in the AV-69 engine: a session that posted or received a tweet shows the tweet emote briefly; attention states win
- [x] Restyle `.peep/graph/index.html` (AV-92) to the graph page design
- [x] Vitest and `cargo test` for the new mood; Shoot both themes

**Done when:** the owner sees the three floppies, the tweet emote on a live exchange, and the restyled graph page.

## Done 2026-11-16

- Floppies: `src/tray.ts` TRAY and `src/bubble.ts` FLOPPIES (cols 15 to 17), `state.rs` `PEEP_SKILLS`; `tray.test.ts`. Shot in the `?demo` page, dark and light: `+5` opens walk, reel, bigpeep, tweet, goose; the first-run disk box now wraps its "+5" line.
- Tweet mood: `mood.rs` `Mood::Tweet` and `tweeting()`, open tweets read from `.peep/tweets.md` by `tickets.rs` `open_tweets`; cargo tests `tweet_while_an_open_tweet_names_the_session_and_clash_wins`, `board_lists_only_open_tweets`. Frontend `emote.ts` row 7, `twice-hold`; demo row `tweets-16` shot in both themes with the call marks in the bubble.
- Graph page: `plugin/skills/goose/scripts/map.mjs` (`areaLinks`, `layoutAreas`, `openTweets`, new `renderPage`), `map.test.mjs`. Shot on this repo and on a 5-area synthetic map with long and back links, dark and light: hover ring and tooltip, pinned panel, Esc clears, no line behind a card.
- `npm run build`, vitest and `cargo test` (392 + 17) pass.
- Not run: the live `tauri dev` exchange. Port 1420 is held by a vite server neither session started (pid 23356), so the widget could not launch; the demo page served by that vite shows this tree.

## Decisions taken without a ruling

- Posted or received means: the tweet's `from` is the session's ticket, or the session edited `.peep/tweets.md` or a file the tweet names in the last 5 minutes (the clash edit window). The emote holds while the tweet is open (SPEC §08.3), not just briefly.
- Tweet sits after greet and before limit sweat, so it also outranks worry; SPEC §08.5 only names sweat, yawn and zzz.
- The graph page's area layout: column is the longest chain of needs above an area (needers left); a link to any column but the next runs in the gap under the target's row, so no line passes behind a card.
- AV-105's report stays above the graph window, as the owner confirmed it; the old file-square board and its file picker are replaced by the area graph.
- Floppy and emote sheets in `src/assets/` replaced by the 18-column and 8-row sheets from `design/skill-pack-2/sprites/`.
