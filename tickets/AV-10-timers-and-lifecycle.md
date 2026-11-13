---
id: AV-10
title: Stuck timer and ended-session fade
status: in-progress
depends: [AV-03, AV-04]
attended: false
---

# AV-10 — Stuck timer and ended-session fade

**What to build:** A backend ticker that feeds `Tick` events into the state engine (stuck after 6 min), and the ended-session behaviour from spec §3.7.

- [x] Tick every 5s; only sessions in `thinking`/`working` are checked
- [x] Ended session: final state `success` if the last turn ended cleanly else `idle`, row dimmed, removed 60s after `endedAt`
- [x] Removal fades out in the UI rather than popping

**Done when:** a killed session dims and leaves after a minute, and a test covers the stuck transition through the ticker.

## Decisions taken without a ruling

- The ticker also ticks `stuck` sessions so "No response for N min" keeps counting; `state.rs` already updated the count on Tick.
- The ticker owns removal: sessions are dropped from the store 60s after `endedAt` (`ticker::tick`), checked on the same 5s pass, so removal lands 60–65s after exit.
- The UI fade is a 600ms opacity animation from the dimmed 0.55 to 0, after which the row is removed; the row's height then collapses without animation.
- A fading row keeps its slot in the list; if the same session id reappears mid-fade the fade is cancelled and the row is reused.
- A hook arriving for an id after its removal creates a fresh placeholder row (existing `apply_or_placeholder` behaviour); not guarded here.

## Owed before done

- Owner check in `npm run tauri dev`: kill a session's terminal, see its row dim, then fade out 60–65s later. Stuck-via-ticker is covered by `ticker::tests::ticker_moves_quiet_thinking_session_to_stuck_at_6_min`.
