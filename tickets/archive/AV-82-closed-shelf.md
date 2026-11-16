---
id: AV-82
title: Closed shelf with resume copy
status: closed
depends: [AV-79, AV-80]
attended: true
---

# AV-82 — Closed shelf

**What to build:** The widget shows sessions that ended in the last 8 hours, so closed work is not forgotten. Per the AV-79 design.

- [x] Render `closed` from the snapshot: peep, last bubble, session name, ticket, end-state chip, flags
- [x] Click copies `claude --resume <id>` with the Jump "copied" moment; no terminal focus (there is none)
- [x] Empty, one, many and long states per the design
- [x] Vitest for ordering and flag rendering; Shoot both themes

**Done when:** the owner closes a session with uncommitted edits, sees it flagged on the shelf, and resumes it from the copied command.

## Done 2026-11-16

- Live in `npm run tauri dev`: throwaway `claude -p` sessions in a scratch repo with an uncommitted file landed on the shelf flagged "uncommitted · no Pack Up", newest first, once their done rows faded. A real click copied `claude --resume e272efbe-…` (clipboard read back), with the accent outline, "On your clipboard!" and the hint bar. Running that resume brought the session back under the same id: it left the shelf, and returned about a minute after it ended.
- Built in `src/closedShelf.ts` (+ `.test.ts`), wired in `src/main.ts`, styles in `src/styles.css`, demo `?demo&closed=none|one|many|long` in `src/demo.ts`. Shot in the browser demo, both themes.

## Decisions taken without a ruling

- End chip text follows the artboard (`ended waiting`, `ended error`, `ended stuck`, else the raw state such as `success`, `idle`), lowercase mono, not the live rows' `STATE_LABEL`.
- A closed entry is hidden while any row with its session id is in the snapshot, fading done rows included, so it lands on the shelf when the live row leaves and leaves again if `--resume` brings back the same id.
- The wave plays as the bubble emote on the copied row; the peep stays `idle` and drops `energy="1"` and the dim/greyscale.
- Footer counts rows whose middle is below the visible bottom (updated on scroll, like the ticket shelf); "to check" there is flagged rows among them; "show all" lifts the five-row cap until reload.
- Header is a button that toggles the shelf; collapsed by default only when empty, the owner's toggle then holds for the session.
- Hint bar reuses `.tray-hint` styling under the shelf; the copied outline and hint both hold 6 s. A failed clipboard write says "Couldn't copy. Type claude --resume <id> in a terminal." like the tray.
- Added `--ps-edge` (design value) for the header's top rule, `--cl-dim` and `--cl-flag` (follows the app's `--ps-accent`, so light is #805300, not the artboard's #8A5A00).
- Time since end: `now`, then `Nm` under an hour, then whole `Nh`.
