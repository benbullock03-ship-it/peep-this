---
id: AV-68
title: Ticket shelf on the open row
status: open
depends: [AV-63, AV-65, AV-67]
attended: true
---

# AV-68 — Ticket shelf

**What to build:** The open row lists the repo's open and parked tickets, per the design.

- [ ] Status chips distinct from session state chips; parked rows dimmed with reason and wake-up
- [ ] Click a ticket: Jump `/peep:ticket <id>` (reuses AV-67)
- [ ] Empty and long shelf per the design
- [ ] Light-theme chip contrast at AA (see AV-52)
- [ ] Vitest for sort (doing, open, parked) and chip mapping; Shoot both themes

**Done when:** the owner has Jumped a ticket from the shelf in `npm run tauri dev`.
