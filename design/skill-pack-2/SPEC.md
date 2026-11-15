# Peep This: skill pack 2 (AV-79)

Design handoff for phase 1: closed shelf, rate-limit pressure on the title, limit meters with numbers, the limit bubble and toasts, the arc line, and three new floppies. It extends `design/shell/` and `design/skill-pack/`; nothing here restyles them. Canvas: https://claude.ai/artifact/FEirV38AYhxsvRYcrBg6gA (private until shared).

## Files

| File | What it is |
|---|---|
| `tokens.css` | Additions to `design/skill-pack/tokens.css`, dark and light. |
| `sprites/floppy-{unpack,walk,reel}-{dark,light}[-disabled].png` | 16 x 16 sprites, same BODY grid and palette rules as `skill-pack/sprites.py`. |
| `ref/*.dc.html`, `ref/pp.css` | Artboard sources from the canvas. They need the Claude Design runtime to render; open the canvas to view. `pp.css` is plain CSS and holds every value used. |

Peeps are the real `<peep-bot>` from `design/peep.js`, unchanged. Sizes stay 48 / 96 / 40 / 32. Floppies stay one 16 px sprite at 16 / 32 / 48 / 96. Motion is stepped; reduced motion shows the last (or first) frame static.

## 01 Closed shelf

1. Sits under the last live row, inside the same scroll, behind a striped header: "Closed · last 8h · N" and, when any, "N to check" in accent.
2. Row: 32 px peep, `idle`, `energy="1"`, `hover="off"`, opacity `--cl-dim` (0.5 dark, 0.55 light), 60% greyscale, on a 4 px plank. Ghost bubble (1 px `--ps-line` outline, muted text) with the last bubble text. Name in IBM Plex Mono, hollow end-state chip, time since end.
3. Third line: ticket stub, then up to three flags in this order: uncommitted, `<id>` left doing, no Pack Up. "Ended waiting / ended error" is the end chip, not a flag.
4. Flags: 1 px `--cl-flag` outline, 4 px square dot, text colour. A flagged row adds a 2 px `--cl-flag` edge to its ghost bubble. No fills, no red, no reorder: newest first always.
5. Empty: one line, "Nothing ended in the last 8 hours." Collapsed by default when empty.
6. Over five rows: scroll inside, footer "N more below · N to check · show all".
7. Click or Enter copies `claude --resume <id>`, reusing Jump: accent outline on the row, the peep wakes (full colour) and plays `wave`, bubble "On your clipboard!", hint bar "Copied claude --resume <id> · now paste it in a terminal" with a Ctrl+V cap, `aria-live="polite"`, 6 s. When the session comes back it leaves the shelf for the live rows.

## 02 Title pressure

1. Driven by the higher of 5h and 7d.
2. Below 80: as today. 80: 2 px `--lim-warn` inset on the title plate, stripes `--lim-warn` at 40%. 95: plate filled `--lim-warn`, `--ps-on-st` text, stripes at a 2 px pitch at 70%. 100: plate filled `--lim-out`, text "Peep This · full", stripes become a 45° hatch (the only diagonal in the shell).
3. Crossing 95 plays one 4-frame nudge, 40 ms each: 0, −2, +2, 0 px. Nothing loops. No glow, no blur.

## 03 Meters with numbers

1. 10 blocks of 3 px + 1 px gap (39 px), then the % in 11 px Plex Mono 600, tabular, min 3ch.
2. Blocks and number: accent / text colour below 70, `--bar-amber` from 70, `--bar-red` from 90.
3. Reset time stays in the hover tooltip.

## 04 Limit bubble and toasts

1. At 80%: every non-attention peep shows `sweat`. The limit takes the verb slot: "AV-81: rate-limit warning · 5h 82%". With no ticket the whole line: "5h limit 82%".
2. An attention state takes the bubble first: filled with its ask, no sweat, no limit text.
3. Toasts at 95% and 100%, once each per limit window. House peep (yellow, no accessory), no buttons; click opens the widget.
   - 95%: "5-hour limit at 95%" / "Resets at 14:20. Anything you start now may stop partway."
   - 100%: "5-hour limit reached" / "New prompts wait until it resets at 14:20, in 1h 12m."
   - Unknown reset: "7-day limit reached" / "Reset time not known yet."

## 05 Arc line

1. One 26 px line on the open row between tray and tags, `--ps-panel-2` fill: star, north star (ellipsis), "N closed · N open · N parked", tick + last closed id. A zero count is dropped.
2. Closed count leads in `--arc-star`, so the number that grows is seen first. Hover or focus shows the full north star and the last ticket's title.
3. On a close, 4 frames at 40 ms: star steps up, count hops 2 px, an accent inset flashes as the new id lands, rest.
4. No north star: "No north star yet ·" muted, then a Jump "The Reel" underlined in `--fl-reel`.
5. Nothing closed yet: "none closed yet" muted in the last slot.

## 06 Floppies

| Name | id | command | dark | light | glyph |
|---|---|---|---|---|---|
| Unpack | `unpack` | `/peep:unpack` | #3FD0D8 | #13959E | arrow up out of a box |
| Walk Through | `walk` | `/peep:walk` | #B6E04A | #76A01A | footprints |
| The Reel | `reel` | `/peep:reel` | #F2A65E | #B8702E | north star |

Hover cards:

- **Unpack.** Opens a session from your Pack Up note: checks the repo still matches, reads back anything that moved, then starts the first step. Use when you sit back down after packing up.
- **Walk Through.** Tours the disks in order, asks a few batched questions about your repo and how you work, then sets the pack up to match. Use the first time, or when the pack feels off.
- **The Reel.** Pins down what the project is for in one line, plus its hook and what it will not do. Use when starting out, or when the work starts to drift.

Glyph rows (10 x 5, `#` ink) for `sprites.py`:

```
unpack: "....##....", "...####...", "#...##...#", "#........#", "##########"
walk:   ".##.......", ".##...##..", "......##..", "..##......", "..##...##."
reel:   "....##....", ".########.", "...####...", "..##..##..", ".#......#."
```
