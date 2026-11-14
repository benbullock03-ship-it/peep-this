---
id: AV-44
title: Pixel desktop shell for the open window
status: in-progress
depends: [AV-26]
attended: false
---

# AV-44 — Pixel desktop shell for the open window

**What to build:** The owner's "Pixel desktop" design (`design/shell/`: `SHELL-SPEC.md`, `shell.css`, `shell.js`, `demo.html`, `ref-C-pixel.png`) applied to the live open window (spec §12.0). The design is the reference: port its markup and CSS into the app's TS modules, keep its look, wire it to the real snapshot. Strip mode is AV-39; light theme is AV-45; window transparency is AV-35.

Owner rulings 2026-11-14: peep sizes 48 (rows), 96 (open stage) as designed; AV-36's 10-block bars stay (mini in the header, full on the open row) instead of the design's smooth bars; square chrome with 6px hard shadow replaces the stair corners.

- [ ] Card: square, 2px border in the text colour, 6px hard shadow, mono type, inside today's window (transparency is AV-35)
- [ ] Header: striped title bar, "PEEP THIS" title, 5h/7d as mini AV-36 block bars, existing header controls (settings button from AV-26) restyled as the design's square icon buttons. The `›` minimise button is left out until AV-39
- [ ] Rows: 48px peep on a striped plank, speech bubble (2-line clamp), session name, outlined state chip. Attention states fill the bubble with the state colour
- [ ] `say` text derived from state and data in a pure module: working "Running {tool}…", waiting the permission prompt/notification text, error the error summary, stuck "zzz… no reply for {since}", success "All done!" or the result summary, thinking the task or a thinking line, idle a resting line
- [ ] Sort: attention states first, otherwise the existing order
- [ ] Open row in place, one at a time: 96px stage with a larger bubble (4-line clamp), tags (time in state, model, kind), last tool, context as a block bar, cwd, then AV-26's last-5 trail. Clicking the head closes it
- [ ] AV-23's approval card still mounts under its row (closed or open) and works with keyboard and mouse as before; pulse on entering `waiting` kept
- [ ] Rows diffed by session id so a snapshot does not restart peep animations or close the open row
- [ ] Dark tokens from `shell.css` `:root` as CSS variables; existing accent and fonts stay bundled locally
- [ ] Demo mode (`src/demo.ts`, browser `npm run dev`) shows the new shell
- [ ] Vitest: `say` derivation per state, attention sort

**Done when:** the owner saw the new shell in `npm run tauri dev` with live sessions, opened a row, and accepted the look.

## Owed before done

- Owner check in `npm run tauri dev` with live sessions: look and bubble wording, open/close a row (a snapshot neither closes it nor restarts peeps), approval card by mouse and keyboard incl. the 1s destructive hold, peep click focuses the terminal, no bottom clipping, 3px header mini bars legible, settings button toggles the log.
- Built and checked in headless Chrome against demo data only (`npm run build`, 69 Vitest tests pass).

## Decisions taken without a ruling

- `say` comes from the snapshot as it is; no Rust change. `state.rs` already writes the notification text (waiting), the error summary (error) and `No response for N min` (stuck) into `task`. Placeholder tasks (`Idle`, `Thinking`, `Working`, `Waiting for input`, `Done`, `Error`) fall back to a line. Waiting with a placeholder task and a held request says "Can I use {tool} on {target}?". Ended sessions say "Session closed. Bye!" (`src/say.ts`).
- Success always says "All done!": the snapshot carries no result summary for a session (only subagents have `result`). Adding one means reading `last_assistant_message` from the Stop hook; left for a follow-up.
- Stuck minutes = 6 min (`STUCK_AFTER_MS`, mirrored from `state.rs`) plus time in state, rather than parsing the task text.
- The three attention states rank equally (as in the design's `sorted`), then newest state change, then id. Before, only `waiting` jumped the queue.
- Header mini bars are 3px square blocks with a 1px gap (39px). AV-36's 4px blocks (58px) overflow the 350px header next to the title and icon button. Lit blocks keep AV-36's neutral/amber/red colours, not the design's accent fill. The open row's context bar uses 8px blocks.
- The closed row has no context bar any more: the design shows context only on the open row, and the ticket says header and open row.
- The card fills the 360px window minus a 6px right/bottom margin so the hard shadow shows inside the opaque window. The body behind is `--ps-desk`. AV-35 makes that margin transparent.
- The subagent count badge stays on the row's peep as a square accent tag. The design has none; it goes when AV-27 child rows land.
- Clicking the peep focuses the terminal (closed row and open stage). The rest of the closed row opens it, and the open head closes it. Name clicks no longer focus, because the name now sits inside the open/close buttons.
- The approval card mounts after the plank, in both closed and open rows, ahead of the open row's facts.
- The design's 600 weights render at 500: Plex Mono ships 400 and 500 only, and no new font files were added.
- The integration banner, notices, approval cards and the settings view move onto the shell tokens with square corners and 2px borders, so nothing keeps the old rounded look.
- Demo mode also mounts the settings view, with an empty log, so the header shows the square icon button there too. Demo tasks are now shaped like `state.rs` output so the bubbles read as they would live.
- `STATE_META` LED colours are gone; state colours are the design's `--st-*` tokens (error `#FF7A7A` and stuck `#A9B3C6` replace `#FF5D5D` and `#8C97AB`). Only the light-theme token block is left out (AV-45), and so are the strip rules (AV-39).
