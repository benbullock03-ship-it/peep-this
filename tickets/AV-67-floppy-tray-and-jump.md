---
id: AV-67
title: Floppy tray on the open row with Jump
status: open
depends: [AV-63, AV-64]
attended: true
---

# AV-67 — Floppy tray and Jump

**What to build:** Start a skill from the widget without typing into the session.

- [ ] Open row shows the floppy tray per the design; keyboard reachable in the design's focus order
- [ ] Jump: copy `/peep:<skill>` to the clipboard, focus the session's terminal (existing `focus_session`), show the "copied, paste in terminal" moment
- [ ] Pack Up action on every open row, separate from the tray, Jumps `/peep:packup`
- [ ] Hover explainer per floppy
- [ ] First-run disk box: shown once, the first time any tray appears; dismissal remembered
- [ ] Vitest for the Jump command builder; Shoot the tray and the disk box

**Done when:** the owner has Jumped a floppy into a live session and pasted it in `npm run tauri dev`.
