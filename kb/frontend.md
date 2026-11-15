# frontend

## How it works now

- The open row hosts the floppy tray and a Pack up button (`src/tray.ts`, wired in `src/rows.ts`); each is a Jump.
- Jump writes the command with `navigator.clipboard.writeText`, then calls `focus_session`; a failed write still focuses and the hint says to type it.
- `jumpCommand(skill, arg?)` builds the command; the hint bar sits in the tray section under the disks.
- The first-run disk box shows once; dismissal lives in `localStorage` `peepthis.diskBox.dismissed`.

## Watch out

- The disk seats only when the snapshot reports the skill; there is no optimistic seating.
- `?demo` (`npm run dev`) is where Shoot runs; real clipboard and terminal focus only work in `npm run tauri dev`.

## Why it's like this

- The widget never types into a session, so Jump copies and focuses for the user to paste.
- Tray contents are the six spec floppies, fixed, until a seventh floppy or a settings ticket exists.

last touched: AV-67, 2026-11-15
