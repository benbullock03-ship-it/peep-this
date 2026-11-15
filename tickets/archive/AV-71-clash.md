---
id: AV-71
title: Clash when two sessions edit the same file
status: closed
depends: [AV-54, AV-63, AV-69, AV-70]
attended: true
---

# AV-71 — Clash

Owner ruling 2026-11-15: ADR 0002 stands over FLOPPY-SPEC ruling 2. Clash is an ambient mood: startled emote, file name and bolt tag on both bubbles, dotted bracket only when the rows are already adjacent; no pin, re-sort or attention fill.

**What to build:** The ambient clash mood from ADR 0002.

- [x] Backend: two different sessions edit the same absolute path within 5 minutes (from AV-54's edit payloads); both get the startled mood with the file name; expires after a short hold
- [x] Frontend: both bubbles show the startled emote and file name, linked per the AV-63 clash design
- [x] No warning, no blocking, no trail entry
- [x] `cargo test`: same file within window, outside window, same session twice (no clash), three sessions

AV-54 finding: absolute path is `tool_input.file_path` for `Edit`, `Write` and `MultiEdit`, but `tool_input.notebook_path` for `NotebookEdit`; `PostToolUse` also echoes `tool_response.filePath`. Count only `PostToolUse` (a `PostToolUseFailure` edit changed nothing). Fixtures: `post_tool_use_edit.json`, `post_tool_use_write.json`, `pre_tool_use_notebook_edit.json`.

**Done when:** two live sessions editing one file show the clash in `npm run tauri dev` and the owner confirms it.

## Done 2026-11-15

Backend: `Event::Edited { path, event }` wraps a `PostToolUse` of Edit/Write/MultiEdit (`file_path`) or NotebookEdit (`notebook_path`) in `ingest/server.rs` (`edited_path`); `Session.edits` keeps the last 5 minutes of edits (`state.rs`); `mood::clash`, `same_path`, `Mood::Startled`, `CLASH_WINDOW_MS` 5 min, `CLASH_HOLD_MS` 30s (`mood.rs`); `refresh_moods` also sets the snapshot's `Session.clash` (absolute path) in `store.rs`. Frontend: `Mood` "startled" and `Session.clash` in `src/model.ts`; `EMOTES.startled` (twice then hold frame 1), `clashTags`, `clashText`, `clashLinks` in `src/emote.ts`; `toRowView(…, clash)` in `src/rowView.ts`; `Bubble.bolt` and the pixel bolt in `src/bubble.ts`; tags and the plank-to-plank bracket (`clash-down`/`clash-up`, `--plank-y`) in `src/rows.ts`; bolt tag, bracket and `twice-hold` styles in `src/styles.css`. Demo: shell-14 and shell-15 clash on `src/shell.css`, adjacent. Tests: mood.rs `two_sessions_editing_one_file_within_5_min_clash_for_the_hold`, `edits_further_apart_than_the_window_do_not_clash`, `one_session_editing_a_file_twice_does_not_clash`, `three_sessions_on_one_file_are_all_startled_and_ended_ones_drop_out`, `priority_is_startled_cheer_greet_worry_yawn_sweat_nap`; server.rs `edit_fixtures_carry_the_absolute_path_and_two_sessions_on_one_file_startle_both` (fixtures through `parse_value` and the store, expiry on tick); `src/emote.test.ts` clash block (tags, text, adjacency, attention, row view). `npm run build` ok, `npm test` 208 pass, `cargo test` 364 + 17 pass. Spec §13.5 and §13.6 updated.

Shoot (`?demo`, 360 and 1280, dark and light): both bubbles read "Both in shell.css" behind an "A" bolt tag with the startled emote (`data-play="twice-hold"`); bubbles unfilled (`rgb(27,25,33)` / `rgb(255,252,246)`), rows not `is-att`; tag `#ffb23e` on ink 9.68:1 dark, `#a85b00` with white 5.05:1 light; text 14.5 / 14.9:1; bracket dotted in `--st-waiting` from shell-14's plank (`--plank-y` 75px) to shell-15's; no text clipped, no page overflow. Screenshots deleted.

## Owed before closed

- The owner's live check in `npm run tauri dev`: two real sessions edit one file, both peeps startle with the file name and bolt tag, and it clears after the hold.

## Decisions taken without a ruling

- Hold is 30s after the later edit (same as a finished subagent's linger); a new edit by either party renews it.
- Startled is the top mood (startled, cheer, greet, worry, yawn, sweat, nap): it is the only mood about two sessions at once, so both bubbles must show it together for the link to read.
- Paths compare ignoring slash direction and ASCII case (Windows); the snapshot names the file by the least spelling any live session used, so all parties carry the same string.
- Three or more sessions on one file are all startled with one tag; the bubble reads "All N in <file>".
- Subagent edits count for their parent session, which is the session that edited.
- The bolt tag sits on an unfilled bubble, so it takes the clash colour (`--st-waiting`) itself rather than the design's inverted tag on an amber fill; the bracket uses the same colour.
- A row in an attention state keeps its ask, shows no tag and is left out of the bracket; its partner keeps the clash bubble and the count.
- Tags A, B… are assigned per render in list order, so a letter can change when an earlier pair expires.
- The strip's bolt and bracket from the design are not built; the ticket asks for the bubbles.
