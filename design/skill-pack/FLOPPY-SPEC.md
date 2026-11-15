# Peep This: floppies, tickets, emotes

Design handoff for the floppy/ticket layer. `ref/*.png` shows every item in dark and light; the numbers match the request list (01 to 15). `tokens.css` holds every value. Sprites are in `sprites/`.

## Rulings

1. **Floppy sizes.** One 16 x 16 sprite, whole-number scales only: 16 on a row plank, 32 in the tray and on the detail-stage plank, 48 in hover cards and first run, 96 as the disk-box hero. No 40: the strip shows no floppy.
2. **Clash link.** Clash is an attention state, so both rows pin to the top and sit together. A dotted bracket in the left gutter joins the two planks, the upper peep looks down and the lower looks up, and both bubbles carry the same bolt tag (A, then B for a second pair). The tag alone is the fallback if the rows cannot be adjacent. Colour: `--st-waiting`.
3. **Ticket status colours.** Ticket chips are hollow stubs with a glyph; session chips stay filled blocks. Blocked is pink, done is grey, so neither borrows a state colour (table below).
4. **Moods per type.** Yes: one master switch plus a switch per emote. Turning a mood off hides the emote only; bubble text and colour never depend on it. Honour `prefers-reduced-motion` by showing frame 1 static.

## Owner rulings (2026-11-15)

- **Pack Up, not Handoff.** Label "Pack Up", command `/peep:packup`, button "Pack up". The art keeps the `handoff` id and filenames.
- **Commands** are the plugin's: `/peep:ducks`, `/peep:qt`, `/peep:ticket`, `/peep:shoot`, `/peep:packup`, `/peep:coderot`. Jump copies e.g. `/peep:ticket AV-34`.
- **Ticket statuses:** files keep AV-56's names; chips show the design's. open → todo, doing → active, done → review (waiting for the owner), blocked → blocked, deferred → parked, closed → done.
- **Explainers** rewritten from AV-57 to AV-61 (table below).
- **Pack Up row action:** every open row also carries a Pack Up button outside the tray (AV-67); not drawn here, use the tray button style.
- Peeps in `ref/` are stand-ins. Use `<peep-bot>`.

## 01 Floppies

| Name | id | command | dark | light | hover card |
|---|---|---|---|---|---|
| Ducks | `ducks` | `/peep:ducks` | #F7D64A | #E3BC2C | Checks where the repo stands: loose tickets, failing tests, stray changes, the last Pack Up note. Use when you sit down, before picking the next ticket. |
| Question Time | `question-time` | `/peep:qt` | #5FCF9A | #2FA873 | Asks you batched questions about a new idea, reads it back, then writes the tickets. Use when the idea is there but the details are fuzzy. |
| Ticket | `ticket` | `/peep:ticket` | #FF8AB8 | #E0609A | Works one ticket end to end: build, check, commit, mark done. Use when a ticket is ready to go. |
| Shoot | `shoot` | `/peep:shoot` | #FF7350 | #E0502F | Checks a web UI change in a real browser with DOM asserts and screenshots. Use after a UI change, before calling it done. |
| Pack Up | `handoff` | `/peep:packup` | #6FA0FF | #3F74E0 | Writes a short note so a fresh session can carry on from here. Use when context is filling up, or you're stopping for the day. |
| Coderot | `coderot` | `/peep:coderot` | #B08AF0 | #8A5FD6 | Breaks code on purpose in a throwaway copy to find what the tests miss. Use between tickets, to check the tests can actually fail. |

- Files: `floppy-<id>-<theme>.png`, plus `-disabled`. `floppies-<theme>.png` is the sheet: 16 px cells, row 0 normal, row 1 disabled, columns ducks, question-time, ticket, shoot, handoff, coderot, blank, then three template examples.
- Render with `image-rendering: pixelated` at 1x, 2x, 3x or 6x.
- **Tray:** 32 px in a 44 px button. **Hover:** up 2 px, `--shadow-lift`. **Focus:** `--bd-focus` on the button. **Disabled:** row 1 of the sheet, not focusable.
- **On plank:** seated at the right end, 2 px in, bottom 5 of 16 rows hidden in the slot (clip, do not scale).
- **Eject:** 5 frames, 80 ms each, `steps()`: seated (11 rows showing), press (8 rows), pop (all 16, up 2 sprite px), hop (up 11, right 4), gone (three dust pixels, disk back in its tray cell).

## 02 Tray

- Open-row order: header, stage, **tray**, tags, last tool, context bar, cwd, trail.
- Six disks show. A seventh cell, `+N`, opens the disk box (5-column grid popover). Tray contents and order come from settings.
- Tab order: row header, seated disk (Delete or Enter ejects), tray, `+N`, cwd, trail. The tray is one tab stop with roving focus: arrows move, Enter or Space inserts.
- A 4 px accent dot under a disk marks the suggested next one in the flow.

## 03 Ticket shelf

- Row: id (600 weight, fixed 46 px), title (ellipsis), status chip right. Min height 40 px.
- Sort: blocked, active, review, todo, parked. Done folds into one "N done · show" line.
- Parked: whole row at 62% opacity, second line "Parked: <reason>. Wakes when <condition>."
- Hover or focus: `--ps-panel-2` fill, accent outline, a Jump button appears left of the chip. Enter on a focused row jumps.
- Empty: Ticket floppy at 32 px, "No tickets yet." / "Pop in the Ticket floppy to write the first one."
- Long: five rows, then the list scrolls inside with a "N more below" footer.

## 04 Ticket status chips

| status | dark | light | glyph |
|---|---|---|---|
| todo | #EDE9F2 | #2A2233 | empty box |
| active | #F7D64A | #8A5A00 | play triangle |
| review | #8FB4FF | #2F5FD0 | eye |
| blocked | #FF8AD1 | #B0237E | no-entry bar |
| parked | #A49FB0 | #645C70 | pause bars, dashed border, row at 62% opacity |
| done | #A49FB0 | #645C70 | tick, label struck through |

Hollow, 2 px border in the status colour, 6 x 4 px notch cut into each side, 14 px glyph, 11 px label. Session chips: solid `--st-*` fill, `--ps-on-st` text, no glyph, no border.

## 05 Bubble

- Grammar: `ID: title · verb` then a 16 px emote slot at the trailing edge. ID 600 weight. Verb at 78% opacity; it is the seated floppy as a verb (shooting, ducking, packing up).
- Truncation: the title ellipsises first. Under about 180 px drop the verb. Under about 90 px show the id only. The id never truncates.
- Attention (waiting, error, stuck, clash): bubble fills with the state colour, the ask replaces the title in 600 weight, the id shrinks to an inverted tab at the start, the verb goes. Normal line returns when the state clears.
- On the 96 px detail stage the bubble may wrap to two lines, verb on the second.

## 06 Emotes

`emotes.json` is the manifest. Sheets: `emotes-on-light.png` (dark theme, because its bubble is light), `emotes-on-dark.png` (light theme), `emotes-mono-ink.png` and `emotes-mono-paper.png` for filled attention bubbles. 8 x 8 cells, 4 frames, 8 fps stepped, 7 rows: startled, sweat, yawn, sparkle, wave, worry, zzz. Drawn at 16 px in bubbles.

## 08 Jump feedback

1. Click: the disk seats on the plank, the button reads "Copied" for 1.2 s.
2. The peep plays `wave`, its bubble says "On your clipboard!", and a hint bar under the row reads "Copied /peep:ticket AV-34 · now paste it in the terminal" with a Ctrl+V key cap. `aria-live="polite"`.
3. The bar stays 6 s or until the session changes state. After three Jumps it shortens to "Copied /peep:ticket AV-34".
4. If the session does not pick the skill up within 60 s, run the eject animation. A seated disk must always mean a running skill.

## 09 Context nudge

- At the threshold (default 80%): `yawn`, dashed `--ps-accent` bubble border, no fill. Copy: "Getting sleepy. I'm 82% full." Buttons: **Pack up** (a Jump for `/peep:packup`) and "Not yet" (snooze to 95%).
- At 95%, once: "Nearly out of room. Pack up now and nothing gets lost."

## 10 to 11 New users

- Hover card: 400 ms delay, also on keyboard focus. 48 px floppy, name, what it does, "Use when ...".
- First run: once, the first time the tray renders. Title "Your disk box", the five flow disks numbered 1 to 5, Coderot set apart as "any time between tickets", one "Got it" button.

## 13 to 14 Templates and packs

- Custom floppy: `{ id, name, command, color, initials | glyph, what, when }`. `glyph` is five strings of ten characters, `#` for ink. `initials` is two letters in the 3 x 5 font. Highlight is the colour mixed 40% to white on column 1, shade 22% to black on column 14 and the label stripe. `sprites.py` in this folder is the reference renderer.
- Emote pack: a PNG in the sheet layout above plus a manifest in the `emotes.json` shape.
- Plank slots: counted from the right. Slot 0 is the floppy; slots 1 and 2 are for sticker packs. 16 x 16 at 1x, 4 px apart, sunk 5 rows. 48 px rows show slot 0 only.

## 15 Settings

Moods (master plus seven), context nudge threshold (50 to 95 in steps of 5, default 80) and "nudge again at 95%", floppies in the tray (tick and drag to order), and a disabled "Per project" block holding colour and accessory for later.
