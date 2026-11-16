---
id: AV-98
title: Hover state on every clickable thing in the widget
status: closed
depends: []
---

# AV-98 — Hover state on every clickable thing in the widget

**What to build:** Every element that does something on click shows it on hover. `src/styles.css` has 20 `:hover` rules today; peeps, chips, bars, strip peeps and tags have none.

- [x] Inventory every clickable element (click handlers in `src/*.ts`) and note which lack `:hover`
- [x] Add a hover state to each in pixel-shell style (`design/shell/`): hard offsets and colour steps, no blur or glow
- [x] Pointer cursor on each; no hover style on things that do nothing when clicked
- [x] Works in light and dark themes; respects reduced motion

**Done when:** in `npm run tauri dev` every clickable element visibly reacts to hover in both themes, and nothing that isn't clickable does. The owner confirms what they saw.

## Inventory

| Element | File | Had hover? | Added |
|---|---|---|---|
| Header icons `.ps-icon` (settings, collapse, strip open) | index.html, stripPlate.ts | yes | - |
| Closed row `.ps-row` (panel and flyout) | rows.ts, flyoutPage.ts | name colour only | panel-2 background |
| Open row head `.ps-openhead` | rows.ts | no | panel-2 background |
| Peep `.ps-peep` (row and stage; click focuses terminal) | rows.ts | no | 2px hop; pointer on the stage |
| Strip cell `.ps-strip-btn` | stripPlate.ts | yes, lost on attention cells | `.is-att` steps 20% to 35% |
| Strip `+N` `.ps-strip-more`, house `.ps-strip-house` (plate click opens) | stripPlate.ts | no | text colour / panel-2 |
| Tray disk, `+3`, overflow grid disk, Pack up | tray.ts | yes | - |
| Disk box Got it `.diskbox-ok` | tray.ts | no | hard lift shadow |
| Arc Reel jump `.arc-jump` | arc.ts | yes | - |
| Ticket shelf row `.shelf-row` | shelf.ts | yes | - |
| End session `.ps-end` | rows.ts | yes | - |
| Nudge Pack up / Not yet | rows.ts | yes | - |
| Kid more/done, shell ended `.ps-kid-toggle` | kidRows.ts, shellRows.ts | yes | - |
| Closed shelf header `.cl-head` | closedShelf.ts | no | top border to text, label plates panel-2 |
| Closed shelf row `.cl-row` | closedShelf.ts | no-op (name already text colour) | panel-2 background |
| Closed shelf show all `.cl-all` | closedShelf.ts | no | panel-2, 2px underline |
| Card buttons `.card-btn`, deny, reason toggle, rule delete, hotkey save, settings change | approvalCard.ts, settings.ts, main.ts | yes | - |
| Allow / Answer in terminal `.card-allow` | approvalCard.ts | brightness filter | hard lift shadow instead |
| Reason chip, rule trail item | approvalCard.ts, rows.ts | yes | - |
| Digest entry and close | digestCard.ts | yes | - |
| Banner action | banner.ts | brightness filter | hard lift shadow instead |
| Banner close, settings close | banner.ts, settings.ts | yes | - |
| Settings toggles `.settings-toggle` | settings.ts | no | accent text |
| Hotkey field `.hotkey-input` (click to record) | settings.ts | no | muted border |
| Not clickable, left without hover: limit meters `.ps-meter`, title, tags, facts, context bar, cwd, stage bubble, chips, shelf head | limits.ts, rows.ts | - | - |

## Decisions taken without a ruling

- Chips, bars and tags have no click of their own; they react through their parent row's hover, not separately.
- The peep keeps its own 2px hop on top of the row hover because its click focuses the terminal instead of opening the row.
- `filter: brightness` hovers on Allow and the banner action became the hard lift shadow the other filled buttons use.
- Strip `+N` and house cells get a hover because a plate click opens the panel.
- Range sliders and checkboxes keep the native hover; text inputs other than the hotkey recorder get none.

## Done 2026-11-16

- Live in `npm run tauri dev`, real CDP mouse hovers in dark and light: row, open head, peep (2px hop), tray disks, `+N`, Pack up, arc Reel Jump, end button, ticket shelf rows, closed shelf head/rows/show all, title-bar buttons and Got it each changed style with `cursor: pointer`; meters, title, arc text, tags, shelf head and stage did not change.
- Light `.cl-head` hover is only the label plates stepping #FFFCF6 to #F1E9DB (its top border is already text colour). Strip and flyout rows were not hovered live; they share the checked rules. Owner closed it as built.
