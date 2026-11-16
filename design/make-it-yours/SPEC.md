# Peep This: Make it yours (settings page)

Design handoff for one scrolling settings page inside the widget panel. It replaces the scattered settings (FLOPPY-SPEC §15's "Per project" block lands here). It extends `design/shell/` and `design/skill-pack/`; nothing here restyles them. Canvas: https://claude.ai/artifact/RfT4cwnXsFzmFMGbMZ9TzH (private until shared), boards 03 to 05.

## Files

| File | What it is |
|---|---|
| `tokens.css` | Additions only, dark and light. |
| `miy.css` | Every rule on the page. Uses tokens only, so it works under `:root[data-theme]` and under the artboards' `.th-dark` / `.th-light`. |
| `demo.html` | Standalone, working, both themes. Serve the repo root (it reads `../peep.js`, the token files, the skill-pack-2 floppy sheets and `src/assets/fonts/`). No network. |
| `ref/MakeItYours.dc.html` | Board 03: the whole page unrolled, default look, repo with setup. |
| `ref/MiyLook.dc.html` | Board 04: look overridden and just saved; accessory not earned with its hint; every switch on, off, focused. |
| `ref/MiyPack.dc.html` | Board 05: repo with no setup; a skill switched off; saved frames; could not save. |

The boards also load `../skill-pack-3/ref/pp4.css` (artboard base). `.dc.html` files need the Claude Design runtime; open the canvas.

## 01 Page

1. Lives in the shell window: 360 px wide, 2 px `--ps-edge` border, `--shadow-hard`. Title bar "Make it yours", sub "saves as you go", back button `‹` at the right returns to the session list.
2. One scroll (`.miy-scroll`, 560 px max like the shell list). Three sections in fixed order: Peep look, Alerts, Pack for this repo.
3. Section header: 30 px, striped like the title bar (4 px pitch, `--ps-line`), 2 px `--ps-edge` above, 2 px `--ps-line` below. Label on a `--ps-panel` plate: IBM Plex Mono 600, 11 px, uppercase, .08em. The right slot is the saved acknowledgement (§05).
4. UI text is IBM Plex Sans 13 px; names, commands, values and labels are IBM Plex Mono. Fredoka is not used inside the panel (the shell title is Mono).
5. No Save, Apply or Cancel button anywhere.

## 02 Peep look

1. Preview: a live `<peep-bot size="96">` on the striped stage with a square bubble, then a 6 px plank with 10 px side margins, as the open row's stage. State `idle`.
2. Source line under the plank, 12 px muted. Default: "Colour and accessory come from this project's folder, so it always looks the same." Overridden: "You picked these for this project." plus a 28 px "Reset to project" button.
3. Colour: a radio group of six 44 px swatches, 6 px apart: "auto" (from project, drawn in the hashed colour with an "auto" plate along its bottom), then yellow, pink, lavender, blue, green (the `peep.js` palette). The legend carries the value: "from project · yellow" or "picked · pink".
4. Selected swatch or cell: border `--ps-text`, 2 px `--ps-panel` gap inside it, and a 10 px `--ps-text` block on the top-right corner. Never colour alone.
5. Accessory: a radio group in a 7-column grid of 44 px cells, 4 px apart (332 px). Cell 1 is "auto"; then the 14 hashed accessories in `identity.ts` order; then the 6 earned ones. Each cell is a 32 px `<peep-bot>` wearing it in the current colour, `hover="off"`, `energy="1"`.
6. Colour and accessory are picked separately; either can stay on auto.
7. Not yet earned: peep at 38% opacity and greyscale, a padlock block (8 x 6 body, 6 x 4 shackle, `--miy-lock`) bottom-right. The cell is focusable with `aria-disabled="true"`; Enter and click select nothing.
8. Hint: on focus, hover or click of a locked cell, one line under the grid: name in 600, "not earned yet", how to earn it, and a 10-block progress bar when it counts. 1 px `--ps-line` outline with a 2 px `--miy-lock` left edge. `role="status"`.
9. After a look change the preview plays `success` for 1200 ms, then `idle`. Reduced motion: stays `idle`.
10. A look change applies to every session in that project at once.

## 03 Alerts

1. Rows are 44 px minimum, full width, separated by 1 px `--ps-line`. The whole row is the control (`role="switch"`).
2. Switch: 36 x 20, 2 px `--ps-edge` border, 12 px square knob 2 px in. Off: knob left, `--sw-knob` on `--sw-track`. On: knob 16 px right, `--sw-knob-on` on `--sw-track-on`. The word "on" or "off" (Mono 500, 11 px, muted) sits left of it.
3. Knob travel: 4 steps over 160 ms. Reduced motion: none.
4. Three switches: "Toast when a peep needs you", "Sound on waiting", "Sound on error". The two sounds are independent.
5. Theme: a two-part segmented control, "dark" and "light", 40 px tall inside a 2 px border (44 px). Selected part is `--ps-text` fill with `--ps-panel` text. It applies at once.

## 04 Pack for this repo

1. First line names the repo: name in Mono 600 13 px, then path and setup file in Mono 11 px muted, ellipsised.
2. One row per skill in sheet order: 32 px floppy, name, command in Mono 11 px muted, on/off word, switch.
3. Switched off: row 1 of the floppy sheet (the disabled art), name in `--ps-muted`, "off here" after the command. The disk leaves this repo's tray and disk box only.
4. Question round size: segmented 2 / 3 / 4.
5. Screenshot widths: chips 36 px tall with a 2 px border, the value and a 32 px remove button; then a 84 px input, placeholder "+ width". Enter adds. 320 to 3840, 5 at most, kept sorted, the last one cannot be removed.
6. No setup yet: the repo line, then one line "No setup in this repo yet." and one button "Set it up". The button is a Jump for `/peep:ducks` (the skill that writes `.peep/setup.md`). Nothing else in the section is drawn.

## 05 Saved

1. Each change writes at once. Two acknowledgements, both inside the widget, no toast:
   - The changed row gets a 2 px `--miy-saved` inset frame for 160 ms.
   - The section header's slot becomes a filled `--miy-saved` plate reading "Saved" with a tick, entering over 4 steps of 40 ms from 4 px above. It holds 1200 ms (`--miy-hold`), then becomes "saved 14:02" in muted text and stays.
2. The slot is `role="status"`, so a screen reader hears "Saved" once per change.
3. Until a section's first change the slot reads "from project" (look) or its last save time.
4. Failure: the control steps back to its old value; the slot is filled `--miy-fail` and reads "Not saved · retry" (`role="alert"`, retry is a button). It stays until a save works. Nothing else is blocked.
5. Reduced motion: the plate appears in place.

## 06 Keyboard and focus

1. Focus is a 2 px `--ps-accent` outline, 2 px outside the control. On full-width rows it is inset 2 px so the scroll edge cannot clip it; on segmented parts it is 4 px out so it clears the border.
2. Tab order: back, colour group, accessory group, Reset (when shown), each Alerts row, theme, each skill row, round size, each width's remove, the add input.
3. Colour, accessory, theme and round size are radio groups: one tab stop, arrows move and select, Home and End jump. The accessory grid also takes Up and Down by 7.
4. Switch rows: Space or Enter toggles.
5. Every control is at least 44 px in one dimension of its hit area; chips and the add input are 36 px (pointer-only secondary controls).

## Tokens

| Token | Dark | Light | Use |
|---|---|---|---|
| `--sw-track` | `--ps-panel-2` | same | switch off |
| `--sw-knob` | `--ps-muted` | same | knob off |
| `--sw-track-on` | `--ps-accent` | same | switch on |
| `--sw-knob-on` | `--ps-on-st` | same | knob on |
| `--miy-saved` | `--ps-accent` | same | saved plate, row frame |
| `--miy-fail` | `--st-error` | same | not saved |
| `--miy-hold` | 1200ms | | saved plate hold |
| `--miy-lock` | #7D7889 | #645C70 | padlock, hint edge |

## Decisions

Picked by the designer where the brief left it open; overrule here.

1. **Saved is shown in the section header**, not as a toast or a floating tick, so it never covers a control and leaves a lasting "saved 14:02".
2. **The row is the switch.** A 36 px track alone is too small a target in a 360 px panel.
3. **"auto" is a choice in each picker**, not a separate mode toggle, so going back to the project look is one click per axis; "Reset to project" clears both.
4. **Locked accessories are visible and focusable** so the hint can be read by keyboard; they cannot be selected.
5. **Hint copy** comes from AV-114's milestone list. The "7 so far" count needs AV-115's local counters.
6. **A switched-off skill stays in the list** in sheet order rather than sinking to the bottom, so the list never reorders under the pointer.
7. **"Set it up" jumps to `/peep:ducks`**, because `.peep/setup.md` says Ducks wrote it. If Walk Through becomes the setup skill, change the command only.
8. **Moods and the context nudge threshold** (FLOPPY-SPEC §15) are not on this page; the brief lists three sections. They need a fourth section or stay where they are.
9. **IBM Plex Mono 600 is not bundled** (`src/assets/fonts/` has 400 and 500). `demo.html` maps 600 to the 500 file.
