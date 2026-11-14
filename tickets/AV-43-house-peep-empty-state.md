---
id: AV-43
title: House peep empty state
status: todo
depends: [AV-39]
attended: false
---

# AV-43 — House peep empty state

**What to build:** With no sessions running the widget shows one mascot peep instead of empty text (spec §12.8). The art is used unchanged.

- [ ] Strip with zero sessions: one 32px peep, `idle` state, yellow, no accessory
- [ ] Panel with zero sessions: the same peep at 64px above "No sessions running"
- [ ] The house peep is not a session: no click-to-focus, no flyout, not counted anywhere
- [ ] It leaves in the same step the first session row enters, and returns when the last ended row has faded
- [ ] The first-run integration banner still shows above it

**Done when:** the owner closed every session and saw the house peep in both strip and panel, then started a session and saw it replaced.
