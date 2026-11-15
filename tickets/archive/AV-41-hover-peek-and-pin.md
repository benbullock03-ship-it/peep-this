---
id: AV-41
title: Hover peek and click pin
status: done
depends: [AV-39]
attended: false
---

# AV-41 — Hover peek and click pin

**What to build:** Hovering the strip shows the panel for as long as the mouse stays; clicking keeps it (spec §12.6).

- [x] Mouse over the strip for 200ms unfolds the panel as a peek
- [x] The peek folds back 300ms after the mouse leaves the panel and strip; re-entering inside that time cancels the fold
- [x] A click anywhere in a peek that is not a row action pins it: mode becomes `panel`
- [x] Row actions work inside a peek (focus a terminal, expand a row) and expanding a row pins it
- [x] A peek never changes the persisted mode and never takes keyboard focus
- [x] No peek while a drag is in progress
- [x] The cursor passing over the strip in under 200ms opens nothing

**Done when:** the owner glanced at the panel by hover and had it fold away untouched, then pinned it with a click, on a right-edge and a top-edge dock.

Decide: whether the 200ms and 300ms delays are settings (recommended: constants; revisit only if the owner finds them wrong).

## Decisions taken without a ruling

- 200ms and 300ms are constants (`OPEN_MS`, `CLOSE_MS` in `src-tauri/src/peek.rs`), as recommended.
- Hover is tracked by the backend, which polls the cursor every 25ms, not by DOM pointer events. "Leaves the panel and strip" means the cursor is outside both the window and the rect the strip had when the peek opened. The transparent corners beside the shadow count as inside.
- A peek pins on the press (`pointerdown`, capture phase), not on the click. So a press that starts a header drag pins too, and a peek can't be dragged and then fold away under the user.
- What does not pin: clicking a row's peep (focus terminal), Allow, Deny and the header collapse button. Everything else pins, including expanding a row, "Deny with reason" (it needs the keyboard), settings and the scrollbar. Scrolling with the wheel does not pin.
- No arming while a mouse button is down, during a native drag, or while a strip/panel switch is animating. After any of those, or after the panel collapses with the cursor resting on the strip (collapse button, hotkey), the cursor has to leave the strip once before a hover can arm. Otherwise a peek would pop open after every drop or collapse.
- `Ctrl+Alt+P` during a peek folds it to the strip, because the panel is what is showing. The tray's "Collapse to strip" also folds it. The inbox hotkey and the tray's settings item pin it. Hiding from the tray during a peek brings the strip back when the widget is shown again.
- While a peek is up, `window.json` keeps `strip` and the strip's own position.
- Fixed in passing: since AV-39, switching to the strip cleared `HOME`, so a floating strip did not go back to its spot when the panel had been pushed inside the work area. Now only an unfold writes `HOME`, and the next strip fit uses it. This applies to both peeks and normal switches.
- A pin that arrives during the unfold animation lets the animation finish instead of cutting it short.
- No peek in a browser (`npm run dev`). There it is strip clicks only, as before.

## Done 2026-11-15

Owner closed it 2026-11-15 on the build installed 19:44. What was owed, as it stood:

Built 2026-11-15. Code: `src-tauri/src/peek.rs` (machine plus 7 tests), `src-tauri/src/window.rs` (`PEEK`, `PEEK_STRIP`, `DRAGGING`, `start_peek`, `end_peek`, `set_shape`, `spawn_peek`, `sys::cursor_over`, `sys::button_down`, `peek` in `ModeView`, `toggled(mode, peek)`), `src/peek.ts` and `src/peek.test.ts` (3 tests), `src/main.ts` (pin listener, `data-peek`).

Seen by Claude 2026-11-15 in `npm run tauri dev` on one 2560×1440 monitor at 100%. The cursor was driven with `SetCursorPos` and `mouse_event`. Shots are in the session scratchpad under `AV-41/`.
- Right edge (column strip at 2486,400): a 120ms pass opened nothing. After 150ms of hover it was still the strip; at 600ms it was the 360-wide panel with the same top and right edge. Notepad, then VS Code, kept the foreground through every peek. The panel stayed open 150ms after the cursor left; returning cancelled the fold, and it was still open 700ms later. It was still open 200ms after leaving and was the strip at 900ms. `window.json` said `strip` throughout.
- Clicking a row's peep in a peek focused VS Code and did not pin: the peek folded and the mode stayed `strip`. Clicking the header blank pinned it: mode `panel`, it stayed open with the cursor away, and the widget took the foreground from the click. Expanding a row in a peek pinned it. Collapsing with `›` left the cursor on the strip, and no peek opened.
- A drag held over the strip for 1.3s opened nothing, and nothing opened after the drop while the cursor stayed there. Leaving and coming back peeked as normal.
- Top edge (row strip at 1214,12): the same hover sequence behaved the same way. The panel unfolded downward from the same top-left, folded back, kept VS Code in front, ignored a peep click, pinned on a header click and collapsed with `›`.

Not seen: a floating strip, a peek while the waiting flyout exists (AV-42 is not built), 150%/200% scaling, reduced motion, the hotkey or tray during a peek (by reading the code only), and how the delays feel to a human hand.

Owner to look at for "Done when", on a right-edge dock and then a top-edge dock: rest the mouse on the strip, and the panel should unfold after a beat without taking the keyboard from the app you were typing in. Move off it, and it should fold back on its own. Hover again and click the panel's header: it should stay open as a pinned panel. Also say whether 200ms and 300ms feel right.
