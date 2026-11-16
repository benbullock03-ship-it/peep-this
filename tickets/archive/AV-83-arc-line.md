---
id: AV-83
title: Arc line on the open row
status: closed
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

## Done 2026-11-16

- Live in `npm run tauri dev` over CDP: `get_snapshot` gave this repo `arc {closed: 92, open: 23, parked: 1, last: AV-92}`, matching a hand count of `tickets/` (14 open + 9 doing, 1 blocked, 92 archived; AV-92 newest). The open row showed "No north star yet · The Reel · 92 closed · AV-92", 26px on `--ps-panel-2`, no overflow; open and parked dropped for width as designed.
- Count-drop order stays as built (owner closed it).
