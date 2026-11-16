# Peep This: Goose report page

Design handoff for the page `/peep:goose` generates: one template, three reports (code map, repo health, custom topic). It is a standalone browser page, not the widget panel. It starts from `design/skill-pack-2/graph.html` and extends SPEC §09 there; nothing here restyles the shell. Canvas: https://claude.ai/artifact/RfT4cwnXsFzmFMGbMZ9TzH (private until shared), boards 06 to 09.

## Files

| File | What it is |
|---|---|
| `map.html`, `health.html`, `topic.html` | The three reports with dummy data. Each is one self-contained file: CSS, JS, fonts, `peep.js` and the floppy sheets are inlined; no network. Open from disk. |
| `template.html` | The one template: frame plus `{{placeholders}}`. |
| `report.js` | Renders any of the three kinds from the JSON in `#report`. |
| `goose.css` | Every rule. Tokens only. |
| `tokens.css` | Additions only. |
| `build.py` | Fills the template three times. Holds the dummy data. Run `python design/goose-report/build.py`. |
| `ref/GooseMap.dc.html` | Board 06, 1280: 12 areas with four sessions, two on one area (dark); 3 areas, no sessions (light). |
| `ref/GooseHealth.dc.html` | Board 07, 1280: several findings (dark); all clear (light). |
| `ref/GooseTopic.dc.html` | Board 08, 1280: long answer (dark); short answer (light). |
| `ref/GoosePhone.dc.html` | Board 09: all three at 375. |

The boards also load `../skill-pack-3/ref/pp4.css`. `.dc.html` files need the Claude Design runtime; open the canvas.

## 01 Frame (all three reports)

1. Page background `--ps-desk`. One Pixel desktop window, 1232 px at most, centred: 2 px `--ps-edge` border, `--shadow-hard`, fill `--go-paper`. Page padding is 2% of the width, 8 px at least, 24 px at most.
2. Title bar: striped as the shell's (4 px pitch), 34 px at least. Left plate: the 16 px Goose floppy, then "Goose · <report name>" in IBM Plex Mono 600 13 px, uppercase, `--ps-accent`. Right plates, Mono 500 11 px: repo (600, `--ps-text`), date (`YYYY-MM-DD`), short commit. The bar wraps; plates never truncate.
3. Tabs under the bar move between the three reports: one row of three equal parts at every width, 44 px tall, Mono 600 12 px, 2 px `--ps-edge` between and below. Current: `--go-paper` fill, `--ps-text`, a 4 px `--ps-accent` top bar, `aria-current="page"`. Others: `--ps-panel-2`, `--ps-muted`. They are links to `map.html`, `health.html`, `topic.html` in the same folder.
4. Lead (map and health): the house peep (yellow, no accessory, 48 px) on a 64 px plank, and a square bubble with the report's one sentence in IBM Plex Sans 14 px. Health: `thinking` with findings, `success` when all clear. Map: `idle`.
5. Footer: 2 px `--ps-edge` above, Mono 11 px muted: the legend (map), what the numbers look back over, "made by /peep:goose".
6. Theme follows `prefers-color-scheme`; there is no toggle. `build.py` moves each token file's light block under the media query.
7. Fonts: Fredoka 600 (topic question only), IBM Plex Sans (prose), IBM Plex Mono (everything else).
8. Layout responds to the width of `.go` (container queries), so the same CSS serves a 375 px column on an artboard and a 375 px viewport.
9. Focus: 2 px `--ps-accent` outline, 2 px out; inset 4 px on tabs.

## 02 Code map

1. **Area**, as skill-pack-2 SPEC §09.2 with a 32 px stand: 120 px wide; 32 px stand, 6 px striped plank, 40 px card (2 px border, no top edge) with name in 600, file count, and a strip of 4 px blocks, one per three files, 16 at most. Blocks for files being edited use `--gr-file-hot`.
2. **Peeps**: live `<peep-bot size="32">` on the plank of the area their session is editing, two at most, 4 px apart. A third or later session is listed at the side only. A waiting peep turns its area's card border `--st-waiting`.
3. **Links**: 2 px, right angles only, `--gr-edge`. A link leaves the right edge of the area that needs at the card's mid-height (57 px down) and ends in a 6 px block at the left edge of the area needed. Each row has its own lane, 12 px apart, starting 12 px into the gutter.
4. **Grid**: column pitch 240 px, row pitch 112 px, 20 px padding, on the 16 px ruled `--ps-desk` ground inside a 2 px border. The box scrolls sideways if the map is wider than the page.
5. **Hover or focus** on an area (areas are buttons): 2 px `--gr-edge-hot` ring, its links turn `--gr-edge-hot`, other links drop to 40%, unlinked areas dim.
6. **Sessions on the map**: a side list, 240 px at least, right of the map at 1280 and under it when narrow. Row: 32 px peep on a plank, name (Mono), "on <area>", filled state chip.
7. **No sessions**: no peeps on the map, and the side list is one outlined line: "No sessions are editing this repo right now. The map still shows what needs what."
8. **Under 720 px** no links are drawn. Each area becomes a row: the area (120 px) on the left; on the right "needs" and "used by", each a row of outlined chips, or "needs nothing" / "used by nothing".
9. Shown at 3 areas (3 x 1) and 12 areas (4 x 3).

## 03 Repo health

1. Four cards in this order: Hot spots, Biggest files, Untested areas, Stale tickets. Then a "Patterns" heading and the pattern cards. Cards sit in a grid of 320 px minimum columns (3 across at 1280, 1 at 375).
2. Card: 2 px `--ps-edge` border, striped 30 px header with the name on a plate and a count plate at the right.
3. Count plate: filled `--go-find` with the number when the card has findings; outlined `--go-clear` reading "clear" when it has none. Fill against outline, so it does not rest on colour.
4. Finding row: 8 px 10 px padding, 1 px `--ps-line` between rows. Line 1: path or name (wraps anywhere), value at the right in 600 with tabular figures. Line 2 (hot spots, biggest files): a 10-block bar, 6 px tall, 200 px at most, relative to the largest in the card; accent below 7 blocks, `--bar-amber` from 7, `--bar-red` from 9. Last line: the reason, 11 px muted.
5. Stale ticket row: id and title, then a hollow 2 px chip in the ticket status colour: "doing · 12 days" (`--tk-active`) or "blocked · 9 days" (`--tk-blocked`).
6. All clear: the card body is one line with a 10 px `--go-clear` block and what was checked ("No file is over 800 lines.").
7. Pattern card: 2 px border on `--ps-panel-2`. The lesson (IBM Plex Sans 600 15 px), one or two sentences tying it to this repo, a fold "Explain like I'm 5" (a `<details>`, 44 px summary, a 16 px `+` / `−` box), and "Source:" with one link in `--go-link`.
8. Each finding group gets at most one pattern card. All clear still shows one.

## 04 Custom topic

1. The owner's question is the page's `<h1>`: Fredoka 600, 22 to 34 px with the width, balanced. Under it, Mono 11 px muted: who asked and how many files and areas the answer drew on.
2. Two columns from 900 px: the answer (68 characters at most) and a 320 px side column. Under 900 the side column drops below the answer.
3. Every answer opens with an "In short" box: 2 px border on `--ps-panel-2`, label in `--ps-accent`, one or two sentences.
4. Answer prose: IBM Plex Sans 15 px / 1.6. Section headings are Mono 600 11 px uppercase muted, numbered "1 · ...". Inline code is Mono 13 px on `--ps-panel-2`.
5. A short answer is the box plus one to three paragraphs with no headings. A long answer has headings and ends with "Read first".
6. Side column: "Files involved", a bordered list of path (Mono) and why (11 px muted); then "On the map", a slice of the code map drawn as §02 at a 164 px column pitch and 104 px row pitch (two columns fit 320 px), with the count ("4 of 12 areas.") and a link to the whole map.
7. Footer says which commit the answer was read from.

## Tokens

| Token | Value | Use |
|---|---|---|
| `--go-paper` | `--ps-panel` | window and card fill |
| `--go-find` | `--st-waiting` | count plate with findings |
| `--go-clear` | `--st-success` | clear plate and block |
| `--go-link` | `--tk-review` | links |

## Decisions

Picked by the designer where the brief left it open; overrule here.

1. **Tabs, not a menu**, for moving between reports: three fixed destinations fit one row even at 375.
2. **No links are drawn under 720 px.** Right-angled links between 120 px areas need gutters a phone does not have; chips say the same thing.
3. **32 px peeps on areas** as the brief says; skill-pack-2 §09 used 48. The house peep in the lead is 48.
4. **Sessions in one repo are drawn with different looks.** Identity hashes the cwd, so that is true for worktrees; sessions sharing one cwd would be identical on the map and are told apart by the side list.
5. **Health order is fixed** rather than worst-first, so the page reads the same every run.
6. **"In short" on every topic answer**, long or short.
7. **System theme only.** The brief says theme follows the system setting, so there is no toggle and the widget's theme switch does not reach this page.
8. **The layout and link router are demo only**, as in `graph.html`: column is depth and links join neighbouring columns. The build brings its own layout.
9. **IBM Plex Mono 600 is not bundled**; 600 maps to the 500 file. IBM Plex Sans 500 is not inlined either (400 and 600 are), to keep each page near 180 KB.
10. **Dummy data**: areas, file names past `state.rs`, counts and ticket titles are invented to look like this repo.
