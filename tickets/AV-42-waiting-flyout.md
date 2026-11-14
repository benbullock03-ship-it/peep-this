---
id: AV-42
title: Waiting row flyout from the strip
status: todo
depends: [AV-39]
attended: false
---

# AV-42 — Waiting row flyout from the strip

**What to build:** When a session enters `waiting` while the widget is a strip, only that session's row slides out beside its peep (spec §12.7). The rest stays collapsed.

- [ ] The flyout is the normal compact row, amber-lit, attached to its peep and pointing away from the snapped edge
- [ ] It leaves when the session's state leaves `waiting`, or when the user clicks it (which focuses the terminal, as a row click does)
- [ ] Two or more waiting sessions each get a flyout, stacked in strip order
- [ ] A waiting session hidden behind the `+N` tile flies out from the tile
- [ ] The toast rules in §3.6 are unchanged; the flyout is not a reason to suppress a toast
- [ ] In `hidden` mode nothing flies out
- [ ] Hovering a flyout does not start a peek; clicking the strip elsewhere still opens the panel
- [ ] The flyout is the mount point AV-23's approval card will use; leave the seam, do not build the card

**Done when:** with the widget collapsed, a live session asked for permission, its row flew out alone, and it went away when the owner answered in the terminal.
