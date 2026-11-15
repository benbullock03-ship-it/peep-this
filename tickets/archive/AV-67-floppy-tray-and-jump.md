---
id: AV-67
title: Floppy tray on the open row with Jump
status: closed
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

## Notes

- Built against AV-63's delivery while AV-63 is still `open`; spec "Owner rulings" text used over the `ref/` PNGs.
- `src/tray.ts` (tray, Pack Up, hover card, disk box, hint; `jumpCommand(skill, arg?)` exported for AV-68), wired in `src/rows.ts`, CSS in `src/styles.css`. Clipboard is `navigator.clipboard.writeText`, written before `focus_session`; no new dependency, no Rust change.
- Shot in `?demo` at 360/1280, dark/light: DOM, contrast (all >= 5.5), focus order, disk box persistence pass. Owed: owner Jumps a floppy into a live session in `npm run tauri dev` (WebView2 clipboard + real terminal focus).

## Decisions taken without a ruling

- Tray holds the six spec floppies in spec order; no `+N`/disk-box popover and no settings-driven contents until a seventh floppy or the settings ticket exists.
- No suggested-next accent dot: nothing yet says which floppy comes next in the flow.
- No optimistic disk seating, peep `wave` or "On your clipboard!" bubble on Jump: the disk seats when the snapshot reports the skill, so the 60s self-eject is not needed; wave is a mood (AV-69 to AV-71).
- No seated-disk tab stop or Delete-to-eject: the widget cannot stop a running skill.
- Tab order is open-row header, disk box "Got it" (while shown), tray (one roving stop), Pack up, then the existing cards.
- Pack up sits after the tray in the same row, styled as a tray button with a 16px Pack Up disk and a border; it wraps under the tray at panel width.
- The hint bar lives in the tray section under the disks, not under the whole row, so it sits where the click happened.
- The three-Jump count is per app run, not persisted.
- Disk box dismissal persists in `localStorage` key `peepthis.diskBox.dismissed` = `"1"`, like the banner dismissal.
- A failed clipboard write still focuses the terminal and the hint says to type the command.
