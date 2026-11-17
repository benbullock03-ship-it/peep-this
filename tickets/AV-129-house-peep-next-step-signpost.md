---
id: AV-129
title: House peep next-step signpost
status: done
depends: [AV-124]
---

# AV-129 — House peep next-step signpost

**What to build:** One signpost that always names the next step, as a Jump (Question Time 2026-11-16). This is the hook: Peep This tells you what to do next.

- [x] The house peep's bubble (empty state) shows one line and its command: integration not installed: "Install the integration"; installed, no session yet: "Start Claude Code in a project"
- [x] A session's row, when `idle` or `success` with no attention state, shows the next step in its bubble: no `.peep/setup.md` in its repo: `/peep:walk`; else the `next:` line from the last Peep skill run in that session (AV-124); else nothing
- [x] Clicking the bubble's command is a Jump: copy it and focus that session's terminal. It never types into the session
- [x] An attention state or a mood takes the bubble first; the signpost never pulses, toasts or sorts the row
- [x] Backend: capture the `next:` line from the skill's final output; `cargo test` with a transcript fixture. Vitest for the bubble's precedence

**Done when:** in a repo with no setup an idle row offers `/peep:walk`; after `/peep:ducks` it offers that run's `next:` command and clicking copies it; the owner confirms in `npm run tauri dev`.

## Checks

1. pass: an idle row in a repo with no setup offers `/peep:walk`. Evidence: vitest `signpost (AV-129)` "offers /peep:walk…"; `.playwright-mcp/av129-demo-dark.png` (walk-17).
2. pass: the `next:` line is captured only after a Peep skill run. Evidence: cargo `transcript_next_line_from_the_last_line_of_main_text_blocks` (fixture `transcripts/next_line.jsonl`), `next_line_needs_a_peep_skill_and_a_new_run_clears_it`.
3. pass in the browser, owed to the owner in Tauri: the row offers the run's command, and a click copies it without toggling the row. Evidence: a Playwright click on ducks-18 `.bb-cmd` left the clipboard holding `/peep:ticket AV-12` with `aria-expanded=false`. Terminal focus only works in `npm run tauri dev`.
4. pass: `/clear, then paste .peep/handoff.md` Jumps `/clear` (the clipboard read back `/clear`) and the bubble shows the whole line; a note shows no chip; `clear to proceed` shows nothing. Evidence: vitest `nextStep`; `av129-demo-dark.png` (packup-19, ticket-20).
5. pass: busy and attention states, a mood, a nudge and a tip all take the bubble first. The signpost touches no sort, toast or pulse code. Evidence: vitest "gives way to…"; diff limited to bubble/rowView/rows.
6. pass: the house shows "Install the integration" with Install when `missing`, otherwise "Start Claude Code in a project" with `claude`. Evidence: vitest `houseSign`; `av129-empty-dark.png`, `av129-empty-light.png`.
7. pass: `Repo.setup` follows `.peep/setup.md`. Evidence: cargo `repo_reports_whether_it_has_a_setup`.
8. pass: build and tests are green. `npm run build`, `cargo check` clean; vitest 375 passed; cargo 425 passed (1 ignored), 17 passed.
9. Live in `tauri dev`, 2026-11-16, run myself over CDP with real `Input.dispatchMouseEvent` clicks. Each session was held open by a stream-json driver in a visible console.
   - pass: in a repo with no setup, a live `success` row showed `Next: /peep:walk` (`Repo.setup` false); a click put `/peep:walk` on the clipboard. Evidence: `.playwright-mcp/live-av129-walk.png`.
   - pass: after a headless `/peep:packup`, the row showed `Next: /clear, then paste .peep/handoff.md` and a click copied `/clear`. Evidence: `live-av129-next.png`.
   - pass: after `/peep:ducks` acted, the row showed `Next: /peep:ticket LO-2` (`Session.next`) and a click copied it. Evidence: `live-av129-ducks.png`.
   - noted: a dead (`alive: false`) row shows "Session closed. Bye!" and no signpost, since there is no terminal to Jump to.
   - not proven: focus. `focus_session` returned `true`, but the foreground window stayed put. A CDP click gives the widget no OS activation, so Windows' foreground lock refuses `SetForegroundWindow`. The trust-prompt keystroke I would have needed for an interactive session was refused by the auto-mode classifier.
   - not run: house Install, because it would rewrite the owner's `~/.claude/settings.json`.

10. (added) Live in `tauri dev`, 2026-11-17 (Big Peep):
   - pass: focus. With Notepad in front, a CDP mouse click on a live row's `Next: /peep:walk` chip put `/peep:walk` on the clipboard and brought that session's Windows Terminal to the foreground. Evidence: scratchpad `av129-jump-focus.png`.
   - pass: house Install on a fresh profile (`CLAUDE_CONFIG_DIR` and `APPDATA` pointed at an empty scratch dir): the bubble read "Install the integration"; Install wrote all 11 hook events and the statusLine wrapper to the scratch `settings.json`, then the bubble read "Start Claude Code in a project" with `claude`. The real `~/.claude/settings.json` md5 was unchanged. The install was started by a physical mouse click at the machine, not the scripted one. Evidence: `av129-house-install-before.png`, `av129-house-install-after.png`.
   - found: the window reads "Not Responding" during Install because `install_integration` runs on the UI thread; filed as AV-148.

## Done 2026-11-17

`src-tauri/src/source/claude/transcript.rs` `TranscriptNext`, `state.rs` `Session.next`, `src/bubble.ts` `signFor`/`nextStep`, `src/house.ts` `houseSign`; tests in checks 1 to 7; live checks 9 and 10.

## Decisions taken without a ruling

- `next:` comes from the transcript (main-thread assistant text blocks whose last line matches), not the Stop hook's `last_assistant_message`, as briefed; a first scan replays nothing.
- "After a Peep skill run" means a Peep skill has run in the session since the widget began tracking it. The line is kept across plain turns, so answers to a read-back still count. It clears when a new Peep skill starts, and a widget restart forgets it.
- Jump rule: a command runs up to the first `,` or `;`, because spaces are part of commands like `/peep:qt <slug>`. The bubble shows the whole line.
- `clear to proceed` shows nothing: the row keeps its normal bubble. A note shows as `Next: <note>` with no chip.
- The signpost also gives way to the context nudge and signal tips, not only to attention states and moods. It replaces the ticket-title bubble on a done row.
- Repo unknown (no cwd) is treated as having a setup: it never offers `/peep:walk`.
- On the closed row the chip is a span inside the row button (buttons cannot nest), so it is mouse-only there. The open row's stage copy is keyboard focusable (`role=button`, Enter/Space).
- House: an `outdated` integration counts as installed, since the banner offers the update. Install calls `install_integration` directly. `claude` is copied only, because there is no session terminal to focus.
