# Peep This: skill pack 2 (AV-79)

Design handoff for phase 1: closed shelf, rate-limit pressure on the title, limit meters with numbers, the limit bubble and toasts, the arc line, and three new floppies. It extends `design/shell/` and `design/skill-pack/`; nothing here restyles them. Canvas: https://claude.ai/artifact/FEirV38AYhxsvRYcrBg6gA (private until shared).

## Files

| File | What it is |
|---|---|
| `tokens.css` | Additions to `design/skill-pack/tokens.css`, dark and light. |
| `sprites/floppy-{unpack,walk,reel}-{dark,light}[-disabled].png` | 16 x 16 sprites, same BODY grid and palette rules as `skill-pack/sprites.py`. |
| `ref/*.dc.html`, `ref/pp.css` | Artboard sources from the canvas. They need the Claude Design runtime to render; open the canvas to view. `pp.css` is plain CSS and holds every value used. |

| `sprites.py` | Generator for sections 07 and 08. Reads the shipped sheets, writes the five floppies, the 18-column sheet, the tweet row and `emotes.json`. |
| `sprites/floppies-{dark,light}.png`, `sprites/emotes-*.png`, `emotes.json` | Supersede the `skill-pack/` sheets and manifest. Existing cells are unchanged. |
| `graph.html` | Section 09 as a standalone page. Needs `../skill-pack/tokens.css`, `tokens.css` and `../peep.js`; serve the `design/` folder to open it. |
| `ref/Floppies5.dc.html`, `ref/TweetEmote.dc.html`, `ref/Graph.dc.html`, `ref/pp3.css` | Artboard sources for 07 to 09. Canvas: https://claude.ai/artifact/J3QvHUUUg4MejSVxFCxUaN (private until shared). |

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

## 07 Five more floppies (phases 2 and 3)

| Name | id | command | dark | light | glyph |
|---|---|---|---|---|---|
| Sneak Peep | `sneak` | `/peep:sneak` | #E060EC | #A82EB5 | an eye |
| Scrubadubdub | `scrub` | `/peep:scrub` | #BFE6F7 | #7CC2E8 | suds |
| Big Peep | `bigpeep` | `/peep:bigpeep` | #5560D8 | #2A2E8C | one big, four little |
| Tweet Tweet | `tweet` | `/peep:tweet` | #DC4667 | #AE2348 | a beak and its call |
| Duck Duck Goose | `goose` | `/peep:goose` | #A47552 | #6F4728 | a V of linked nodes |

Hover cards:

- **Sneak Peep.** Mocks up two or three versions of a screen on a local server, side by side, before anything is built. You pick one. Use when you can only say what it should look like once you see it.
- **Scrubadubdub.** Finds the AI-made tells in a UI, plus messy spacing and text, says where each one comes from, and fixes only the ones you approve. Use when a screen works but looks generated.
- **Big Peep.** Runs a batch of tickets, one little peep per ticket, and checks on any that go quiet. Use when several tickets are ready and none needs you in between.
- **Tweet Tweet.** Tells the other sessions on this repo that this one is stuck, or is about to touch their files. Use when another session may know the answer, or before changing a file one of them is in.
- **Duck Duck Goose.** Maps what depends on what across the codebase and draws it as a graph. Use when you need to know what a change will touch.

1. **Colour rule.** Every hue was already taken by the nine, so the five differ in lightness as well as hue. `sprites.py` prints the closest pair of all 14 per theme (CIE76): it is still Pack Up and Coderot, 25.0 dark and 24.7 light. A new floppy must not go under that.
2. **Sheet.** `floppies-<theme>.png` is now 18 columns: 0 to 9 as shipped (the six, blank, three templates), 10 to 12 unpack, walk, reel, 13 to 17 sneak, scrub, bigpeep, tweet, goose. Row 0 normal, row 1 disabled.
3. States, sizes, plank seating and eject are FLOPPY-SPEC's, unchanged.

## 08 Tweet emote

1. Row 7 of the emote sheets, `emotes.json` name `tweet`. 8 x 8 cell, 4 frames, 8 fps stepped. Ink beak at the left, call marks in `--em-tweet` going right: shut, open with both marks, shut with the far mark, open with the near mark.
2. Plays twice, then holds frame 1. Reduced motion shows frame 1.
3. Shown when the session has posted or received a tweet that is still open. Same emote both ways; the bubble line does not change.
4. It is a mood: the state never changes, and an attention state takes the bubble first (filled, its own mono emote, no tweet). The tweet returns when the state clears.
5. Against clash (`startled`): startled is upright, centred, bounces, tinted `--em-startled`; tweet is sideways, anchored left, opens and shuts, tinted `--em-tweet`. If one peep has both, clash takes the slot. Tweet outranks sweat, yawn and zzz. This order is the designer's call; overrule it here.

## 09 Graph page

1. A Pixel desktop window: 2 px `--ps-edge` border, `--shadow-hard`, striped title bar "Peep This · graph" with repo, area, link and peep counts.
2. **Area.** 120 px wide: a 48 px stand, a striped plank, then a 40 px card (2 px border, no top edge) with the name in 600 weight, the file count, and one strip of 4 px blocks, one per three files, 16 at most. Blocks for files being edited use `--gr-file-hot`.
3. **Peeps.** Live `<peep-bot size="48">` on the plank of the area their session is editing, two at most side by side. A waiting peep turns its area's border `--st-waiting`.
4. **Links.** 2 px, right angles only, `--gr-edge`. A link leaves the right edge of the area that needs, and ends in a 6 px block at the left edge of the area needed. Each row has its own lane in the gutter so runs from different areas do not merge.
5. **Hover or focus.** The area gets a 2 px `--gr-edge-hot` ring, its links turn `--gr-edge-hot`, other links drop to 40% and unlinked areas dim (card stays opaque so no line shows through). A tooltip card gives name, path, files, needs, used by.
6. **Selected.** Click pins the area: card fills `--ps-panel-2` and a 280 px panel opens on the right with path, Needs and Used by as chips that move the selection, Peeps here (48 px, name, chip, bubble) and open tweets. Esc clears.
7. Shown at 5 areas (3 x 2) and 20 areas (5 x 4). The column layout and router in `graph.html` are demo only and handle links between neighbouring columns; the build brings its own layout.

## New tokens (07 to 09)

`--fl-sneak`, `--fl-scrub`, `--fl-bigpeep`, `--fl-tweet`, `--fl-goose` (values above). `--em-tweet` is `--em-dream`. `--gr-edge` and `--gr-file` are `--ps-edge`; `--gr-edge-hot` and `--gr-file-hot` are `--ps-accent`.

Glyph rows:

```
sneak:   "..######..", ".#..##..#.", "#...##...#", ".#..##..#.", "..######.."
scrub:   ".##....#..", "#..#..#.#.", "#..#...#..", ".##..##...", ".....##..."
bigpeep: ".####.....", "######.#.#", "######....", "######.#.#", ".####....."
tweet:   "#......#..", "###..#..#.", "####.#..#.", "###..#..#.", "#......#.."
goose:   "##......##", "###....###", "..##..##..", "...####...", "....##...."
```
