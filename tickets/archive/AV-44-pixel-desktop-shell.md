---
id: AV-44
title: Pixel desktop shell for the open window
status: done
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

## Follow-up queued (owner, 2026-11-14, after first live look)

- [x] Bash bubble: use the tool's `description` when present ("Running the test suite…"); otherwise program plus first argument ("Running python…", "Running npm test…"). Never a heredoc, pipe or full command in a bubble. Full command stays on Last tool and in the approval card. If `description` is not in the snapshot, add it minimally in Rust with a test.
- [x] Last tool on the open row: end-ellipsis (no middle cut), wraps to 2 lines, full text on hover.
- [x] Header mini bars: AV-36's 4px blocks; drop the gap between label and bar to make room; title unchanged. If 4px still overflows, show blocks only and move the % to a tooltip.

## Done 2026-11-14

Live check by Claude at the owner's request, in `npm run tauri dev` with live and placeholder sessions, screenshots in the session scratchpad:

- **Look:** shell, planks, square bubbles, chips, 96px stage, facts and trail render as designed over dark and light backdrops.
- **Open row:** survives 3 snapshots and stays open, with no peep node recreated.
- **Approval cards:** work by mouse and by keyboard, including the 1s destructive hold. See the AV-24, AV-26 and AV-28 checks.
- **Peep click:** with Notepad in front, clicking this session's peep focused its VS Code window.
- **Bottom:** no clipping. The last row ends 10px above the list bottom, and the card plus shadow fit the window.
- **Header:** blocks show without the %; hover reads "5h 7% · 7d 20% · updated just now", and `aria-label`s carry the %.
- **Settings:** the button toggles the log.

Follow-up, not blocking: the description-to-gerund rule misfires. "Screenshot …" becomes "Screenshoting…" (missing from `DOUBLED`). "White topmost backdrop …" becomes "Whiting…", because a description that opens with a non-verb gets treated as a verb. Owner rulings on header % and the success result line are still open.

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
- Follow-up: the Bash `description` was not in the snapshot. `PreToolUse` now carries it for Bash/PowerShell only (first line, clipped) and it rides on `lastTool.description` for sessions and subagents. Other tools' descriptions stay out (the Agent one is already the target).
- Follow-up: the description is turned into a doing phrase ("Run the test suite" -> "Running the test suite…") with a small gerund rule and a list of short verbs that double their last letter. Without one, the bubble says "Running {program} {first plain arg}…": first segment of the command, split at unquoted `|`, `&`, `;` and newlines, everything after a redirect or heredoc dropped, `cd` segments, `VAR=x` and `sudo`/`env`/`time` prefixes skipped, program reduced to its basename without `.exe`. Flags, `$` and quoted prose are not shown as the argument; `run`/`exec`/`x`/`dlx`/`-m` pull in one more argument ("npm run build", "python -m pytest").
- Follow-up: waiting now prefers the oldest held request over the notification text, so "Needs permission: Bash <full command>" never reaches the bubble. Shell asks read "Can I {description}?" or "Can I run {short}?", using the classifier's flagged span when there is one, so `git fetch && git reset --hard` asks "Can I run git reset?" rather than about the harmless first step. Without a held request, a `Needs permission:` task for a shell tool is rebuilt the same way from `lastTool`. Child bubbles use the same code.
- Follow-up: Last tool shows the full text from the snapshot, clamped to 2 lines with an end ellipsis (`word-break: break-all`, so line 1 is filled before wrapping), with the full text as the hover title. `truncateMiddle` is gone. state.rs still cuts targets over 60 chars in the middle; the hover shows that 60-char text, not the raw command.
- Follow-up: 4px blocks with AV-36's 2px gap and no label gap still overflow with the % beside them (about 211px of meters for about 185px free), so the header shows blocks only and the hover title reads "5h 42% · 7d 18% · updated just now". Each meter has an `aria-label` with its %. Measured in headless Chrome at 360px: header scrollWidth equals clientWidth (350), usage 162px.
- Follow-up: the demo's "Find the line item schema" child now runs `.venv/Scripts/python - <<'EOF'` so the browser demo shows the short Bash bubble.
