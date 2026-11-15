---
id: AV-43
title: House peep empty state
status: in-progress
depends: [AV-39]
attended: false
---

# AV-43 — House peep empty state

**What to build:** With no sessions running the widget shows one mascot peep instead of empty text (spec §12.8). The art is used unchanged.

- [x] Strip with zero sessions: one 40px peep, `idle` state, yellow, no accessory (was 32px; see Decisions)
- [x] Panel with zero sessions: the same peep at 96px above "No sessions running" (was 64px; see Decisions)
- [x] The house peep is not a session: no click-to-focus, no flyout, not counted anywhere
- [x] It leaves in the same step the first session row enters, and returns when the last ended row has faded
- [x] The first-run integration banner still shows above it

**Done when:** the owner closed every session and saw the house peep in both strip and panel, then started a session and saw it replaced.

## Decisions taken without a ruling

- **Sizes 40px strip / 96px panel, not 32/64** (orchestrator ruling in the run). The owner's standing rule allows only 32, 40, 48 and 96. 40 is the strip peep size, so the plate is the same size with the house as with one session; 96 is the open-row stage size. Checklist above and spec §12.8 changed to match.
- **Copy is "No sessions running"** as the ticket and spec say; it replaces the old "No Claude Code sessions running".
- **Panel look:** peep centred on a 120px plank (the row `.ps-plank` stripes) with the text under it. The shell spec lists the empty state as not designed, so this is the nearest thing in the design language, not a new style.
- **Strip look:** a non-button cell with the session cell's footprint and shelf plank, no hover fill. A press on it does what a press anywhere on the plate does (opens the panel); it never focuses a terminal.
- **Hover:** the panel house keeps peep.js's hover reaction; the strip house has `hover="off"` like strip cells.
- **Return timing:** the panel shows the house once no session is in the snapshot (ended ones count until the backend drops them) and no row is still fading. Strip cells have no fade-out, so the strip house returns in the same render the last cell leaves.
- **Peek (AV-41):** the house sits outside `.ps-item`, so a press on it during a peek pins like any other panel press; it is not in `KEEPS_PEEK`.
- **Browser check param:** `?empty` shows the empty state, `?empty=banner` adds the install banner, `?empty=cycle` brings one demo session in and out every 3.2s. Combine with `&strip=column|row`.

## Owed before done

- Seen (headless Edge on `npm run dev`): `?empty` panel with the 96px house and text; `?empty=banner` with the install banner above it; `?empty&strip=column` and `strip=row` with the 40px house; `?empty=cycle` with the session row in place of the house, the house absent while the last row fades, and back after; strip plate the same size with the house as with one session.
- Seen (`npm run tauri dev`, 2 real sessions): panel rows as before, no house peep.
- Not seen: the live widget with zero sessions (real sessions were running), the fold animation into or out of an empty strip, and light theme.
- Owner: close every session and look at the house in strip and panel, then start one and watch it replaced. Judge the panel composition (centred peep on a short plank), since the empty state was never designed in `design/shell`.
