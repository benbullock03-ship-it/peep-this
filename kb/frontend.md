# frontend

## How it works now

- The open row hosts the floppy tray and a Pack up button (`src/tray.ts`, wired in `src/rows.ts`); each is a Jump.
- Jump writes the command with `navigator.clipboard.writeText`, then calls `focus_session`; a failed write still focuses and the hint says to type it.
- `jumpCommand(skill, arg?)` builds the command; the hint bar sits in the tray section under the disks.
- The tray shows six disks, then a `+N` cell (`TRAY_SHOWN` in `src/tray.ts`) that opens the rest in a grid under the row, its own roving tab stop. Sprite sheets in `src/assets/floppy/` are the 18-column sheets from `design/skill-pack-2/sprites/` (16px columns, CSS width 288 units); `FLOPPIES` in `src/bubble.ts` maps skill to column and bubble verb. The tray lists 10: Core (walk, qt, ticket, shoot, packup, ducks), then Extras (sneak, goose, coderot, bigpeep) behind `+4`. Tweet is inserted-only: a `FLOPPIES` column and a `PEEP_SKILLS` entry, no `TRAY` entry. Hover-card `when` lines are the tour's "use when" text (`tray.test.ts` checks them against `tour.md`). A new floppy needs a `TRAY` entry, a `FLOPPIES` column and a `PEEP_SKILLS` entry in `state.rs`, or it never seats. Every disk is always enabled: no locked or dimmed state (progressive unlocks were removed).
- The first-run disk box shows once; dismissal lives in `localStorage` `peepthis.diskBox.dismissed`.
- The ticket shelf (`src/shelf.ts`) sits after the tray and lists the repo's tickets from `Snapshot.repos`, passed as `render(sessions, repos?)` in `src/main.ts`; a click Jumps `/peep:ticket <id>` through the tray's hint bar (`createTray().fire`).
- The arc line (`src/arc.ts`) sits between tray and shelf: north star from `.peep/reel.md`, closed/open/parked counts and the last closed id, all from `Repo.northStar` and `Repo.arc` (`src-tauri/src/tickets.rs`). Counts that do not fit drop whole from the end; closed always stays. No north star shows a Jump to `/peep:walk`.
- Shelf sort: blocked, doing, done (shown as review), open, deferred (parked); file order within a status. `closed` never reaches the shelf.
- The closed shelf (`src/closedShelf.ts`, mounted in `src/main.ts` after the list) renders `Snapshot.closed` newest first; a click copies `claude --resume <id>` with no terminal focus. An entry hides while its id is live again. Demo states: `?demo&closed=none|one|many|long`.
- End session: a "Hold to end session" button on the open row only, wired by `bindHold` (`src/hold.ts`, shared with the destructive Allow) to `end_session`; a failure shows in the bubble for 6s.
- A session's mood shows as a 16x16 emote in the bubble's trailing slot (`src/emote.ts` `MOOD_EMOTE`, `emoteFor`; `setEmote` in `src/rows.ts`). Sheet follows the bubble fill: dark theme takes `emotes-on-dark.png` from `src/assets/emotes/` (8 rows; row 7 is `tweet`, which plays twice then holds frame 1).
- Tweet mood (`mood.rs` `tweeting`): an open tweet on the repo's `.peep/tweets.md` whose `from` is the session's ticket, or a recent edit of the board or of a file the tweet names. It holds while the tweet is open; clash outranks it, it outranks sweat, worry, yawn and nap.
- Rate limits: header meters with % on a second header line and title pressure at 80/95/100 (`src/limits.ts`); from 80% every non-attention peep sweats and the limit takes the bubble's verb slot. Toasts at 95 and 100 come from the backend (`src-tauri/src/limit.rs`), once per window keyed by `resets_at`. `?demo&limits=82,30` fakes them.
- The context nudge (`nudgeFor`, `nudgeText`) shows a dashed bubble with Pack up and Not yet under the plank; the threshold is the Settings "Context nudge" slider (`nudge_pct`/`set_nudge_pct`). It rides the yawn, so it shows only with Settings "Show moods" on.
- Learned page (`src/learned.ts`) opens with "Your rhythm": Pack Up sessions, tickets closed through `/peep:ticket` and widget approvals, each with its first-use day, from `learned().rhythm` (`learned.rs` `Rhythm`, stored as `learned.rhythm` in the widget `settings.json`). "Copy my rhythm" copies `rhythmSummary` for a beta issue; nothing is ever sent.
- Clash: sessions with `mood: startled` and the same `Session.clash` path get a lettered bolt tag (`Bubble.bolt` in `src/bubble.ts`), "Both in <file>" bubble text (`clashTags`, `clashText` in `src/emote.ts`) and, when adjacent, a dotted plank-to-plank bracket in `--st-waiting` (`clash-down`/`clash-up`, `--plank-y` in `src/rows.ts`).

- Identity (`src/identity.ts`): the cwd hashes over `COLORS` and the 14 `ACCESSORIES`; the six `EARNED_ACCESSORIES` (packed, clear, ten, spotless, architect, supporter) are never hashed and reach the picker only through `pickerAccessories(unlocked)`. `?accessories` (`src/accessories.ts`) renders every accessory for checks.
- Own your peep: the open row's "Change peep" picker (`src/picker.ts`) writes an override through `set_peep_look`, stored as `peepLooks` in the widget `settings.json`, keyed by `cwdKey` (FNV-1a of the normalised cwd, 8 hex; `look.rs` `cwd_key` matches). `createLookStore` (`src/look.ts`) resolves the override over the hash for row, strip, children (colour only), closed shelf (`forCwd`) and, through the `peep-looks` event, the flyout. A pick equal to the hash is dropped, so picking the hash back is a reset.
- Toast icons: `src/toastIcon.ts` draws each live project's `waiting` peep and hands it to `set_toast_icon` (`%APPDATA%/PeepThis/icons/<key>.png`); the waiting toast uses it as its app logo. The goose page reads the same `peepLooks`.

- Signpost (AV-129): an idle or done row with no mood, nudge or tip shows `Next: <cmd>` (`signFor`, `nextStep` in `src/bubble.ts`): `/peep:walk` when `Repo.setup` is false, else `Session.next`. A click on the `.bb-cmd` chip Jumps (copy, then focus) without toggling the row; only the stage copy is keyboard focusable. A command's Jump stops at the first `,` or `;`; a note shows no chip, and `clear to proceed` shows nothing.
- The house bubble (`houseSign` in `src/house.ts`): `missing` shows Install (the `install_integration` call); otherwise it shows `claude`, which is copied only because there is no terminal to focus.

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
- A floppy sprite must be a single `span` in its parent: CSS that targets the text span in a disk row (e.g. `.diskbox-rot > span:last-child`) must not match the disks, or the whole sheet shows.
- The Learned page reads the rhythm when it opens; reopen it to see a new count. A ticket close counts only if a session linked to that ticket is live across the shelf reads before and after the archive.
- First-use dates wrap mid-date at the 360px panel width.
- Not yet snoozes in memory per session until 95% or the yawn clears; it does not survive a restart.

- Every clickable has a `:hover` (hard offset, colour step or 2px hop, never blur, glow or brightness filter) and `cursor: pointer`; things that do nothing on click have neither. Chips, bars and tags react through their row. New clickables need both.

## Why it's like this

- The widget never types into a session, so Jump copies and focuses for the user to paste.
- Tray cells are 40px wide, not 44, so six disks plus `+N` fit the 312px row.
- A Jump click needs real user activation: a click fired from page JS gets "Couldn't copy". Live checks click over CDP `Input.dispatchMouseEvent`.
- Ticket chips (2px hollow stub, notch, glyph, lowercase) differ in shape from session chips so the two never read as the same thing.
- Nudge buttons sit under the plank because the closed row is one `<button>` and cannot hold buttons.
- The bolt tag sits on an unfilled bubble and carries the clash colour itself; clash is a mood, never an attention fill (ADR 0002).

last touched: AV-137, 2026-11-17
