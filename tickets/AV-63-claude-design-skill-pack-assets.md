---
id: AV-63
title: Claude Design deliverables for floppies, shelf, chips and emotes
status: open
depends: []
attended: true
---

# AV-63 — Claude Design deliverables (owner)

**What to deliver:** The owner brings these from Claude Design as source (HTML/CSS/JS), like `design/shell/`. They go in `design/skill-pack/` and become the reference, not to be restyled.

- [x] Floppy set: six skills plus a blank template; tray, hover, focus, inserted, eject frames, disabled; light and dark; sizes ruled
- [x] Floppy tray placement in the open row, overflow and focus order
- [x] Ticket shelf row: six status chips distinct from the seven session state chips, parked rows dimmed with reason and wake-up, empty and long shelf
- [x] Bubble format for ticket + skill + state, truncation, attention takeover
- [x] Emote sheet: startled, sweat, yawn, sparkle, wave, worry, zzz
- [x] Clash link between two rows
- [x] Jump "copied, paste in terminal" moment
- [ ] Context nudge bubble with a Pack Up Jump; a Pack Up action on every open row (nudge drawn; row action not drawn, spec says reuse the tray button style)
- [x] Floppy hover explainers and the first-run disk box
- [x] Token sheet covering all the above

**Done when:** the files are in `design/skill-pack/` and the owner says they're final.

**Delivered 2026-11-15** as `FLOPPY-SPEC.md`, `tokens.css`, `emotes.json`, sprite PNGs, `ref/` mocks and `sprites.py` (reference renderer), not HTML/CSS/JS like `design/shell/`. Owner rulings that day: Pack Up over Handoff, `/peep:*` commands, chip-to-file status mapping, nudge at 80% then 95%, explainers rewritten from AV-57 to AV-61. All recorded under "Owner rulings" in the spec. The `ref/` PNGs still show the old Handoff copy and explainers; the spec text wins.
