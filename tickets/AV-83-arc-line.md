---
id: AV-83
title: Arc line on the open row
status: doing
depends: [AV-65, AV-75, AV-79]
attended: true
---

# AV-83 — Arc line

**What to build:** One line per repo on the open row showing where the project stands, per the AV-79 design.

- [x] Backend reads `north_star` from `.peep/reel.md` alongside the tickets it already reads (AV-65)
- [x] Counts closed / open / parked tickets and names the last ticket closed
- [x] Frontend renders the line with and without a north star
- [x] `cargo test` for the reel read and counts; Shoot both themes

**Done when:** the owner sees this repo's arc line on an open row in `npm run tauri dev`.

## Decisions taken without a ruling

- `open` counts open, doing and done (done still waits for the owner); `parked` counts blocked and deferred; `closed` is archive files plus live `status: closed`, deduplicated by id.
- The last ticket closed is the closed file (archive or live) with the newest mtime, ties to the highest id; only that one archive file is read, for its title.
- The reel is read on the tickets thread's 5s poll; `.peep/` is not watched.
- The line sits directly after the tray, before the ticket shelf (both are between tray and tags).
- Counts that don't fit the 26px line drop whole from the end (parked, then open) via a wrapping, clipped counts box; the hover/focus tip lists every count. Closed always shows.
- With no north star, "No north star yet ·" keeps its width over open/parked and ellipsises only after they have dropped; "The Reel" Jump sits right after it.
- The close animation (§05.3) plays when the open row's repo reports a new last id; a row reopened after a close plays it once too.
- The tip shows on hover or keyboard focus (`:focus-visible`), not on mouse-click focus.

## Owed before done

- Owner sees this repo's arc line on an open row in `npm run tauri dev` (expected: no north star, "The Reel", 92 closed, open count, last AV-92). A dev widget started by another session was running without CDP, so the `peepthis: snapshot` line and a webview shot could not be taken.
- Judgement: with three counts the north star shrinks to a few characters at the widget width ("A calm …"); the owner may prefer counts to drop first.
