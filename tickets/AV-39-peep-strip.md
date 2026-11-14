---
id: AV-39
title: Peep strip collapsed state
status: todo
depends: [AV-35, AV-37, AV-38]
attended: false
---

# AV-39 — Peep strip collapsed state

**What to build:** A second visible form of the widget: a slim plate holding one 40px peep per session (spec §12.5).

> Owner ruling 2026-11-14: build the strip from the AV-44 design (`design/shell/`: `stripHTML`, `.ps-strip`, `.ps-strip-btn`): 62px plate, 40px peeps on mini shelves, attention cells tinted and ringed, a state dot per cell, the `‹` expand button at the top. Where the list below says 32px or stairs, the design wins. This ticket builds the strip and the switch between strip and panel; the hotkey (AV-40), hover peek (AV-41) and waiting flyout (AV-42) come after.

- [ ] Backend holds the mode: `panel`, `strip` or `hidden`; persisted; the app launches in `strip`
- [ ] Strip plate has the same border and hard shadow as the card, no border on the docked edge
- [ ] One 40px peep per top-level session in the panel's sort order, with its real state, colour and accessory. No names, bars, LEDs, subagents or shells
- [ ] More than 8 sessions: 8 peeps, then a `+N` tile in pixel type
- [ ] A failed shell's red dot shows on its parent's peep once AV-30 exists (leave the hook, do not build shells here)
- [ ] Orientation follows the anchor from AV-38: a column on the left and right edges, a row on the top and bottom edges, a column at corners and when floating
- [ ] Strip and panel share the anchor: the panel unfolds from the strip away from the snapped edge, in steps, and the anchored edge never moves
- [ ] The whole plate drags the widget; a press that does not move is a click
- [ ] Clicking the strip switches to the panel; a tray menu item switches back until AV-40 lands
- [ ] Peeps keep animating in the strip and stop when the mode is `hidden`
- [ ] Vitest for the strip's session selection and `+N` count

**Done when:** the owner switched between strip and panel on a right edge, a top edge and floating, with nothing jumping, and restarted into the strip.
