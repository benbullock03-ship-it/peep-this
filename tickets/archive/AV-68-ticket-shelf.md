---
id: AV-68
title: Ticket shelf on the open row
status: closed
depends: [AV-63, AV-65, AV-67]
attended: true
---

# AV-68 — Ticket shelf

**What to build:** The open row lists the repo's open and parked tickets, per the design.

- [ ] Status chips distinct from session state chips; parked rows dimmed with reason and wake-up
- [ ] Click a ticket: Jump `/peep:ticket <id>` (reuses AV-67)
- [ ] Empty and long shelf per the design
- [ ] Light-theme chip contrast at AA (see AV-52)
- [ ] Vitest for sort (doing, open, parked) and chip mapping; Shoot both themes

**Done when:** the owner has Jumped a ticket from the shelf in `npm run tauri dev`.

## Built 2026-11-15

`src/shelf.ts` (sort, chip mapping, parked line, shelf DOM), `src/shelf.test.ts`; shelf mounted after the tray in `src/rows.ts`, Jumping through the tray's hint bar (`createTray().fire`, `src/tray.ts`); `render(sessions, repos?)` takes `Snapshot.repos` (`src/main.ts`); `--tk-*` tokens and shelf styles in `src/styles.css`; demo repos in `src/demo.ts` (digest-04 normal, sync-05 long 17, research-02 empty, triage-03 no repo). Built against AV-63's `FLOPPY-SPEC.md` while AV-63 is still `open`.

Shoot (`?demo`, dark and light, 360 and 1280): chip text on the panel, dark min 6.76 (parked), light min 5.59 (review); on the hover fill `--ps-panel-2`, dark 5.98, light 4.75 (review). Parked/blocked note text dark 6.76, light 6.20. Long shelf shows 5 rows, "12 more below · scroll", clears at the end; click AV-36 put `/peep:ticket AV-36` on the clipboard and the hint bar read "Copied /peep:ticket AV-36 · now paste it in the terminal". No page overflow. `npm run build`, `npm test` (200) pass.

## Decisions taken without a ruling

- **Sort follows the design, not the ticket line:** blocked, active (doing), review (done), todo (open), parked (deferred); file order within a status.
- **No "N done · show" fold:** `closed` never reaches the shelf (AV-65) and `done` shows as review, so there is nothing to fold.
- **Parked dims by colour, not 62% opacity:** id and title go `--ps-muted`, chip and note stay full colour; 62% opacity put parked text near 2.7:1 in light and 3.4:1 in dark, under AA.
- **Blocked tickets also show their reason line** ("Blocked: <reason>. Wakes when <wake>."), at full strength; the design draws it only for parked.
- **Shelf sits right after the tray** (design gives no slot); the hint bar stays in the tray, above the shelf.
- **Session chips left as they are** (1px outline plus dot, uppercase); ticket chips are 2px hollow stubs with a notch, glyph and lowercase label, so they differ in border, shape and glyph. Design ruling 3's filled session chip is not built here.
- **No repo for a session (old backend or no cwd): shelf hidden.** A repo with zero tickets shows the empty shelf.
- **Each ticket row is its own tab stop** (a button; Enter jumps); the design gives no roving rule for the shelf.
- **Glyphs are 7 x 7 pixel SVGs at 2x**, drawn here; no glyph sprites in `design/skill-pack/sprites/`.
