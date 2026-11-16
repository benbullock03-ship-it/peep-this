# Peep This: skill pack 3 (insert, drive light, step progress)

Design handoff for what a seated floppy does between insert and eject. It extends `design/skill-pack/FLOPPY-SPEC.md` §01; the eject frames, the sprites and every existing token are unchanged. Canvas: https://claude.ai/artifact/RfT4cwnXsFzmFMGbMZ9TzH (private until shared), boards 01 and 02.

## Files

| File | What it is |
|---|---|
| `tokens.css` | Additions only, dark and light. Load after the two earlier token files. |
| `drive.css` | Working CSS for the seated disk, insert, drive face, pips and step label. Every number below is in it. |
| `sprites/drive-{dark,light}.png` | 16 px cells: 0 led off, 1 led on, 2 pip to do, 3 pip done, 4 pip current, 5 empty tray slot. |
| `sprites/drive-face-{dark,light}.png` | Two cells: the 26 x 4 face at 1x with the light on and six steps, for reference. |
| `sprites.py` | Generator for both. |
| `ref/Main.dc.html` | Board 01: insert frames, light states, 48 px rows, finished. Both themes. |
| `ref/DriveSteps.dc.html` | Board 02: 2 to 6 steps at both sizes, open row, step change, step names. Both themes. |
| `ref/pp4.css` | Artboard base (tokens scoped per theme). Shared by the `make-it-yours` and `goose-report` boards. |

The `.dc.html` files need the Claude Design runtime; open the canvas to view them.

"Sprite px" below means one pixel of the 16 px floppy: 1 px on a 48 px row, 2 px on the open row's stage (`--px`).

## 01 Insert

1. Five frames, 80 ms each (`--dv-frame`), 400 ms, `steps(1, end)`. The eject poses run backwards.
2. Frame 1, lift: the disk in its tray cell rises 2 px with `--shadow-lift` (the tray hover pose). Nothing on the plank yet.
3. Frame 2, hop: the disk appears over the plank, all 16 rows showing, up 18 and right 4 sprite px from its seat. The tray cell switches to the empty slot (rule 7).
4. Frame 3, pop: all 16 rows, up 7 sprite px.
5. Frame 4, press: down 3 sprite px, 8 rows showing.
6. Frame 5, seated: FLOPPY-SPEC §01 (right end, 2 px in, 11 rows showing). The drive face appears on this frame with the light on and step 1 current.
7. While a disk is out, its tray cell shows `drive-<theme>.png` cell 5 at 32 px (dotted floppy outline), is not focusable and takes no click.
8. Reduced motion: frame 5 at once.

## 02 Drive face

1. Drawn only while a disk is seated. A plank with no disk is plain.
2. A `--dv-face` bar cut into the plank: 26 sprite px wide, 4 px tall, top 1 px below the plank's top, right edge 1 px in from the plank's right end. The plank stays 6 px tall at both sizes, so `--px` scales the face's widths only.
3. The face is the same width for any step count, so the light never moves.

## 03 Drive light

1. One lamp at the face's left: 2 sprite px wide, 2 px tall, 2 sprite px from the face's left edge, 1 px from its top.
2. Off (`--dl-off`): the disk is seated but the session has not picked the skill up yet (the 60 s window in FLOPPY-SPEC §08), and during eject.
3. On (`--dl-on`): the skill is running. Steady in every session state except `working`.
4. Blinking: only while the session state is `working`. A 480 ms loop (`--dl-blink`) in three 160 ms steps: on, on, off.
5. Reduced motion: steady on.
6. The light is not a state. The chip and bubble still carry state; attention states still take the bubble.

## 04 Step progress

1. One pip per step, 2 to 6 steps. Pips are 2 sprite px wide and 2 px tall with a 1 sprite px gap, right-aligned 1 sprite px in from the face's right edge and 1 px up from its bottom. Six pips are 17 sprite px, which sits under the 16 px disk.
2. To do: `--sp-todo`. Done: `--sp-done`. Current: `--sp-now` and 3 px tall (it rises 1 px to the face's top edge). The three differ in lightness and shape, not hue alone.
3. Row (16 px floppy): pips 2 x 2, gap 1. Stage (32 px floppy): pips 4 x 2, gap 2.
4. Step change, 4 frames of 40 ms (`--sp-frame`): the old pip settles to done at 2 px; the next pip takes `--sp-now` at 2 px; it overshoots to 4 px; it rests at 3 px. Reduced motion: the end state at once.
5. A skill that declares no steps, or one step, shows the light and no pips.
6. Step name, open row only: IBM Plex Mono 500, 10 px / 12 px, right-aligned 1 px in from the plank's right end, 9 px below the plank's top (in the tray's top padding). Format `3/5 asserts`: count in `--ps-text` 600 with tabular figures, 6 px gap, name in `--ps-muted`. Names are 10 characters at most; a longer one is cut to 9 plus an ellipsis. An unnamed step shows the count alone.
7. A 48 px row never draws the name. The plank's accessible label carries it: "Shoot, step 3 of 5: asserts".

## 05 Finished

1. When the last step completes, every pip is done, the light is steady on, and the open row's label reads `5/5 done` with the count in `--dl-on`.
2. Hold 1200 ms (`--dv-hold`).
3. Then the light goes off and the existing eject runs (FLOPPY-SPEC §01, 5 frames, 80 ms each). The pips clear on its pop frame; the face is removed on its last frame, when the disk is back in its tray cell.
4. Reduced motion: hold 1200 ms, then remove disk and face in one step.
5. A skill that ends early (error, user stop) skips the hold: light off, eject.

## Tokens

| Token | Dark | Light | Use |
|---|---|---|---|
| `--dl-off` | #4A4458 | #C9BFAE | lamp unlit |
| `--dl-on` | #7DFFA6 | #1F7A3D | lamp lit, finished count |
| `--sp-todo` | #4A4458 | #C9BFAE | pip not started |
| `--sp-done` | `--ps-text` | `--ps-text` | pip done |
| `--sp-now` | `--ps-accent` | `--ps-accent` | pip current |
| `--dv-face` | `--ps-panel` | `--ps-panel` | face fill |
| `--dl-blink` | 480ms | | blink loop |
| `--sp-frame` | 40ms | | step-change frame |
| `--dv-frame` | 80ms | | insert frame |
| `--dv-hold` | 1200ms | | finished hold |

## Decisions

Picked by the designer where the brief left it open; overrule here.

1. **Progress lives in the plank, not on the disk.** A seated disk shows only its shutter and 2 label rows; drawing on it would mean new floppy art per step.
2. **The plank does not grow on the stage.** Pips double in width only (4 x 2), so the 6 px plank from the shell is kept.
3. **Lamp colour is the success green**, not the accent, so it cannot be read as a seventh pip. It is 4 sprite px away from the nearest pip.
4. **Insert is 5 frames.** The app's current `floppy-insert` in `src/styles.css` is 4 frames over 320 ms with no lift; the lift frame makes it a true mirror of the 5-frame eject.
5. **Empty tray slot** while a disk is out, so a disk is never in two places.
6. **Blink is 2:1 on:off** so the lamp reads as lit at a glance; a 1:1 blink read as a warning.
7. **Hold is 1200 ms**, the same as the Jump "Copied" label in FLOPPY-SPEC §08.
