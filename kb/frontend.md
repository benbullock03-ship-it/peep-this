# frontend

## How it works now

- The open row hosts the floppy tray and a Pack up button (`src/tray.ts`, wired in `src/rows.ts`); each is a Jump.
- Jump writes the command with `navigator.clipboard.writeText`, then calls `focus_session`; a failed write still focuses and the hint says to type it.
- `jumpCommand(skill, arg?)` builds the command; the hint bar sits in the tray section under the disks.
- The tray shows six disks, then a `+N` cell (`TRAY_SHOWN` in `src/tray.ts`) that opens the rest in a grid under the row, its own roving tab stop. Sprite sheets in `src/assets/floppy/` are 16px columns; `FLOPPIES` in `src/bubble.ts` maps skill to column and bubble verb.
- The first-run disk box shows once; dismissal lives in `localStorage` `peepthis.diskBox.dismissed`.
- The ticket shelf (`src/shelf.ts`) sits after the tray and lists the repo's tickets from `Snapshot.repos`, passed as `render(sessions, repos?)` in `src/main.ts`; a click Jumps `/peep:ticket <id>` through the tray's hint bar (`createTray().fire`).
- Shelf sort: blocked, doing, done (shown as review), open, deferred (parked); file order within a status. `closed` never reaches the shelf.
- The closed shelf (`src/closedShelf.ts`, mounted in `src/main.ts` after the list) renders `Snapshot.closed` newest first; a click copies `claude --resume <id>` with no terminal focus. An entry hides while its id is live again. Demo states: `?demo&closed=none|one|many|long`.
- A session's mood shows as a 16x16 emote in the bubble's trailing slot (`src/emote.ts` `MOOD_EMOTE`, `emoteFor`; `setEmote` in `src/rows.ts`). Sheet follows the bubble fill: dark theme takes `emotes-on-dark.png` from `src/assets/emotes/`.
- Rate limits: header meters with % on a second header line and title pressure at 80/95/100 (`src/limits.ts`); from 80% every non-attention peep sweats and the limit takes the bubble's verb slot. Toasts at 95 and 100 come from the backend (`src-tauri/src/limit.rs`), once per window keyed by `resets_at`. `?demo&limits=82,30` fakes them.
- The context nudge (`nudgeFor`, `nudgeText`) shows a dashed bubble with Pack up and Not yet under the plank; the threshold is the Settings "Context nudge" slider (`nudge_pct`/`set_nudge_pct`).
- Clash: sessions with `mood: startled` and the same `Session.clash` path get a lettered bolt tag (`Bubble.bolt` in `src/bubble.ts`), "Both in <file>" bubble text (`clashTags`, `clashText` in `src/emote.ts`) and, when adjacent, a dotted plank-to-plank bracket in `--st-waiting` (`clash-down`/`clash-up`, `--plank-y` in `src/rows.ts`).

## Watch out

- The disk seats only when the snapshot reports the skill; there is no optimistic seating.
- `?demo` (`npm run dev`) is where Shoot runs; real clipboard and terminal focus only work in `npm run tauri dev`. Demo repos live in `src/demo.ts`.
- Parked tickets dim by colour (`--ps-muted`), not opacity: 62% opacity fell under AA. Ticket chip tokens are `--tk-*`.
- A session with no repo hides the shelf; a repo with zero tickets shows the empty shelf.
- Attention rows hide the emote; the mood is still in the snapshot. An attention row also drops out of a clash: no tag, no bracket; its partner keeps the clash bubble.
- Clash letters are assigned per render in list order, so a letter can change when an earlier pair expires. The strip shows no clash.
- Emotes loop at 8 fps per `emotes.json`, not the 160ms transition rule; sparkle and wave play twice then hide while the mood holds.
- The shelf header click sets a sticky open/closed choice for the session; a stray click on it collapses a full shelf until reload.
- Busy sessions post real statusline payloads every second or so, overwriting a faked one; a live limit check reposts in a loop while it samples.
- Not yet snoozes in memory per session until 95% or the yawn clears; it does not survive a restart.

## Why it's like this

- The widget never types into a session, so Jump copies and focuses for the user to paste.
- Tray cells are 40px wide, not 44, so six disks plus `+N` fit the 312px row.
- A Jump click needs real user activation: a click fired from page JS gets "Couldn't copy". Live checks click over CDP `Input.dispatchMouseEvent`.
- Ticket chips (2px hollow stub, notch, glyph, lowercase) differ in shape from session chips so the two never read as the same thing.
- Nudge buttons sit under the plank because the closed row is one `<button>` and cannot hold buttons.
- The bolt tag sits on an unfilled bubble and carries the clash colour itself; clash is a mood, never an attention fill (ADR 0002).

last touched: AV-82, 2026-11-16
