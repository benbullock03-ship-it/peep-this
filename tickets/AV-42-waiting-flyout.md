---
id: AV-42
title: Waiting row flyout from the strip
status: in-progress
depends: [AV-39]
attended: false
---

# AV-42 — Waiting row flyout from the strip

**What to build:** When a session enters `waiting` while the widget is a strip, only that session's row slides out beside its peep (spec §12.7). The rest stays collapsed.

- [x] The flyout is the normal compact row, amber-lit, attached to its peep and pointing away from the snapped edge
- [ ] It leaves when the session's state leaves `waiting`, or when the user clicks it (which focuses the terminal, as a row click does)
- [x] Two or more waiting sessions each get a flyout, stacked in strip order
- [ ] A waiting session hidden behind the `+N` tile flies out from the tile
- [x] The toast rules in §3.6 are unchanged; the flyout is not a reason to suppress a toast
- [ ] In `hidden` mode nothing flies out
- [x] Hovering a flyout does not start a peek; clicking the strip elsewhere still opens the panel
- [x] The flyout is the mount point AV-23's approval card will use; leave the seam, do not build the card

**Done when:** with the widget collapsed, a live session asked for permission, its row flew out alone, and it went away when the owner answered in the terminal.

## Decisions taken without a ruling

- **A second window, not a wider strip window.** The flyouts live in a transparent, non-focusable, always-on-top window (`flyout`) over the zone beside the strip, with its hit region cut to the cards. Hovering a flyout is outside the strip's window, so AV-41's peek never sees it without any change to `cursor_over`; and the strip's window never resizes, so no WebView2 blank frame and no plate jump. Cost: a second WebView2 renderer, created hidden at startup.
- The zone spans the work area's height beside a column (316px wide) and its width off a row, out to the far edge. Only the cards and their tails take clicks; the rest passes through.
- Side: right-edge and right-corner strips fly left, left ones fly right, top flies down, bottom flies up. A floating strip flies right like the panel, unless only the left has room.
- Beside a column each flyout is level with its peep and pushed down past the one before (6px apart); a stack that would run off the bottom is lifted. Off a row each sits under (or over) its own peep and they stack away from the strip in strip order.
- The card is the panel's compact row (48px peep, bubble, name, chip, plank) on an AV-44 card 300px wide, not the panel's 354. "Amber-lit" is what a waiting row already has (amber bubble and chip, the 1.5s inset pulse) plus an 8px stepped amber tail pointing at the peep; the card border and shadow stay neutral, per §3.1. Only the flyout nearest a row strip has a tail.
- Entrance: slides 16px out from the strip side and fades in over the standard motion (5 steps / 160ms), clipped by the region; instant under reduced motion. Leaving is instant.
- A click anywhere on the card except the future `.cards` host focuses the terminal. Only when a window came forward is the flyout dismissed, until that session's next `waiting` spell (keyed on `stateSince`); a failed focus flashes the row and keeps it, as the panel does.
- Which sessions fly out is decided in the strip's page (it knows the cells); the flyout page lays them out. Flyouts are hidden while the strip is hidden, peeking, switching, being dragged, or not yet strip-shaped after a switch.
- Ended sessions never fly out, even if their last state was `waiting`.
- The `design-tropes` skill is not installed here; the look was checked against `design/shell/` and the panel's own row, by screenshot.

## Owed before done

Built 2026-11-15. Code: `src-tauri/src/flyout.rs` (window, side and zone maths, `set_flyouts` / `get_flyouts` / `flyout_region`, 6 tests), `src-tauri/src/window.rs` (`still_strip`, refresh on `mode`, move and resize; `set_region`, `place_rect`, `show_inactive` made crate-visible), `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json` (the `flyout` window), `flyout.html`, `vite.config.ts` (second page), `src/flyout.ts` + `src/flyout.test.ts` (10 tests: selection, dismissal, `+N` mapping, stacking, sides), `src/flyoutPage.ts`, `src/stripPlate.ts` (`slotCenter`), `src/main.ts` (`syncFlyouts`), `.ps-fly` in `src/styles.css`, spec §12.7. Checks: `npm run build`, `npm test` 148, `cargo test` 290 + 15.

Seen by Claude 2026-11-15 in `npm run tauri dev` (one 2560×1440 monitor at 100%), with synthetic `Notification` hooks for fake session ids (placeholder sessions, no terminal). Shots in the session scratchpad under `AV-42/`.
- Right edge (column): one waiting session flew out alone to the left of the strip, level with its ringed peep, amber tail at the peep. A second stacked under it in strip order. A `UserPromptSubmit` for the first took its flyout away and left the second.
- Hovering a flyout for 1.2s opened no peek (the strip stayed a strip; only the bubble's tooltip showed).
- Top edge (row): two flyouts stacked downward, each under its own peep, the first with an upward tail. Clicking a non-waiting cell opened the panel and the flyouts went; `Ctrl+Alt+P` folded back and they returned. A hover peek hid them and they came back when it folded.

Not seen live, so not claimed:
- Clicking a flyout through to a real terminal and the flyout leaving: the fake sessions have no window, so focus failed and the flyout stayed, as designed. The dismissal path is code only.
- `hidden` mode (the tray was not driven by script): guarded in `still_strip`.
- The `+N` tile with more than 8 sessions (unit-tested only), a floating, left or bottom strip, a drag with flyouts up, 150%/200% scale, reduced motion, light theme.
- A real permission prompt; the toast was not watched.

Owner to look at for "Done when": with the widget collapsed on a right edge, let a live session ask for permission. Its row should fly out alone beside its peep, and go away when you answer in the terminal. Also click a flyout once and check it focuses the terminal and leaves.
