---
id: AV-60
title: Pack Up (from handoff) and Shoot in the pack
status: open
depends: [AV-56]
attended: false
---

# AV-60 — `/peep:packup` and `/peep:shoot`

**What to build:** Move the two global skills into the pack, generic and lean. The global `handoff` becomes Pack Up.

- [ ] Pack Up: carries the session forward into a new one. Note under 25 lines, directive, no background; names the ticket in progress, parked tickets touched, and the next command to run
- [ ] Pack Up asks 2 or 3 batched questions (recommended answers first) only when the next direction is unclear; otherwise writes straight away
- [ ] Pack Up saves to `.peep/handoff.md` (overwriting the last one), copies it to the clipboard, then says it's safe to `/clear` or exit; Ducks reads that file on its next run (AV-58)
- [ ] Usable any time the user chooses to end a session, not only at high context
- [ ] Shoot: web UIs only; dev URL and start command come from the repo setup; DOM asserts plus screenshots judged against questions written first; both themes and widths from the setup when present
- [ ] Strip anything specific to this repo or to other projects (datamap etc.)
- [ ] Note in the owner's global skills that the pack versions exist; don't delete the global ones

**Done when:** both run from the plugin in a repo other than this one.
