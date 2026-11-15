# frontend

## How it works now

- The open row hosts the floppy tray and a Pack up button (`src/tray.ts`, wired in `src/rows.ts`); each is a Jump.
- Jump writes the command with `navigator.clipboard.writeText`, then calls `focus_session`; a failed write still focuses and the hint says to type it.
- `jumpCommand(skill, arg?)` builds the command; the hint bar sits in the tray section under the disks.
- The first-run disk box shows once; dismissal lives in `localStorage` `peepthis.diskBox.dismissed`.
- The ticket shelf (`src/shelf.ts`) sits after the tray and lists the repo's tickets from `Snapshot.repos`, passed as `render(sessions, repos?)` in `src/main.ts`; a click Jumps `/peep:ticket <id>` through the tray's hint bar (`createTray().fire`).
- Shelf sort: blocked, doing, done (shown as review), open, deferred (parked); file order within a status. `closed` never reaches the shelf.

## Watch out

- The disk seats only when the snapshot reports the skill; there is no optimistic seating.
- `?demo` (`npm run dev`) is where Shoot runs; real clipboard and terminal focus only work in `npm run tauri dev`. Demo repos live in `src/demo.ts`.
- Parked tickets dim by colour (`--ps-muted`), not opacity: 62% opacity fell under AA. Ticket chip tokens are `--tk-*`.
- A session with no repo hides the shelf; a repo with zero tickets shows the empty shelf.

## Why it's like this

- The widget never types into a session, so Jump copies and focuses for the user to paste.
- Tray contents are the six spec floppies, fixed, until a seventh floppy or a settings ticket exists.

- Ticket chips (2px hollow stub, notch, glyph, lowercase) differ in shape from session chips so the two never read as the same thing.

last touched: AV-68, 2026-11-15
