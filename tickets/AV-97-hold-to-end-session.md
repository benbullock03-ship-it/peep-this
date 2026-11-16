---
id: AV-97
title: Hold to end a session on the open row
status: doing
depends: [AV-96]
---

# AV-97 — Hold to end a session on the open row

**What to build:** An "end session" control on the open row that needs a 1s hold, the same gesture as destructive approvals (AV-22).

- [x] Button on the open row only: never in the strip, flyout or toast
- [x] 1s hold fills the button stepped (motion per AV-37; instant fill under reduced motion), and releasing early cancels
- [x] On completion call `end_session`; show the failure in the bubble if it returns an error
- [x] Pixel-shell style per `design/shell/`, light and dark themes
- [x] Vitest for the hold timer (complete, cancel)

**Done when:** in `npm run tauri dev`, holding the button on a windowless session (e.g. a Claude desktop scratch session) ends it and the row fades; a short click does nothing. The owner confirms what they saw.

## Notes

- `src/hold.ts` `bindHold` is the AV-22 pointer/keyboard hold wiring lifted out of `src/approvalCard.ts`; both the destructive Allow and `.ps-end` use it. `src/hold.test.ts` covers complete, early-release cancel, keyboard hold and refused start.
- The button lives in the open row's facts (`createFacts` in `src/rows.ts`), which are detached when the row closes; strip, flyout page and toasts never build it. Live wiring: `invoke("end_session", { sessionId })` in `src/main.ts`.
- Demo: `?demo` ends the session (row goes not-alive, button hides); `?demo&end=fail` rejects with AV-96's mismatch message.
- Shoot (headless Chromium, 375, dark and light): no `.ps-end` before opening, one inside `.ps-item.is-open .ps-facts`, none in strip elements; at 450ms into a hold `::before` is `scaleX(0.4)` with `transform 1s steps(10)`; release at 500ms clears it and the session stays alive; a full hold sets `data-alive=false` and hides the button; reduced motion is `scaleX(1)` at 100ms with transition `none`; `end=fail` puts "Could not end: the PID now belongs to another process" in the bubble on `--st-error`.
- `kb/frontend.md` not updated (outside this run's files): it could note `bindHold` and the end button.

## Owed before done

- Owner live check in `npm run tauri dev`: hold the button on a windowless throwaway session; it ends and the row fades; a short click does nothing.

## Decisions taken without a ruling

- Fill is 10 steps over the 1s hold (one per 100ms, like the 10-block bars); AV-37's 4 to 6 frames over 160ms is for transitions, too fine to read on a 1s hold. The destructive Allow fill stays linear, untouched.
- Under reduced motion the fill is instant but the hold still lasts 1s; the gesture is the safety, not the animation.
- The button sits right-aligned at the bottom of the open row's facts, under the cwd, styled like the destructive Allow (2px `--red` border, transparent, `--red` 35% fill), label "Hold to end session".
- A failed end shows "Could not end: <error>" in both bubble copies on `--st-error` for 6s (the tray's hint time), then the snapshot's text returns; snapshot updates do not overwrite it meanwhile.
- `Ok(false)` (already exited) is treated as success: the lifecycle finishes the row. After success the button shows "Ending…" disabled, and resets after 6s if the row is still there.
- The button hides when the session is not alive.
- `src/main.ts` was touched to pass the `end_session` binding; `createList` gained a seventh `onEnd` parameter and builds the button only when it is given.
