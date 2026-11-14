---
id: AV-41
title: Hover peek and click pin
status: todo
depends: [AV-39]
attended: false
---

# AV-41 — Hover peek and click pin

**What to build:** Hovering the strip shows the panel for as long as the mouse stays; clicking keeps it (spec §12.6).

- [ ] Mouse over the strip for 200ms unfolds the panel as a peek
- [ ] The peek folds back 300ms after the mouse leaves the panel and strip; re-entering inside that time cancels the fold
- [ ] A click anywhere in a peek that is not a row action pins it: mode becomes `panel`
- [ ] Row actions work inside a peek (focus a terminal, expand a row) and expanding a row pins it
- [ ] A peek never changes the persisted mode and never takes keyboard focus
- [ ] No peek while a drag is in progress
- [ ] The cursor passing over the strip in under 200ms opens nothing

**Done when:** the owner glanced at the panel by hover and had it fold away untouched, then pinned it with a click, on a right-edge and a top-edge dock.

Decide: whether the 200ms and 300ms delays are settings (recommended: constants; revisit only if the owner finds them wrong).
