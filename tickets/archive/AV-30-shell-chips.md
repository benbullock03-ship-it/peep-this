---
id: AV-30
title: Shell chip rows
status: done
depends: [AV-29, AV-28]
attended: false
---

# AV-30 — Shell chip rows

**What to build:** Stub. A chip row per background shell under the parent, below any child peeps (spec §11.7).

- [x] Chip: `>_` glyph, command (mono, middle-truncated), live runtime, state label; `serving` shows the port
- [x] No peep on shell rows
- [x] `failed` puts a red dot on the parent row that stays visible when children are collapsed; hover shows exit code and last line
- [x] `exited` chips linger 30s then fold, like finished subagents
- [x] Vitest: formatting and fold timing

**Done when:** the owner saw a dev server chip showing its port and a failed build flagged on a collapsed parent.

## Built

- `src/shells.ts` (pure): `middleTruncate`, `charsThatFit`, `shellRuntime`, `shellLabel`, `shellTitle`, `planShells`, `nextShellFoldAt`, `failNote`, `SHELL_LINGER_MS` (30s). Tests in `src/shells.test.ts` (15).
- `src/shellRows.ts`: chip rows diffed by shell id, a ResizeObserver that middle-truncates the command to the width it has, a 1s runtime tick while any shell is live, and a `>_ ended N` toggle for folded chips.
- `src/rows.ts`: a `.ps-shells` host after the subagents; a `.ps-fail` red dot on the parent's peep with the failure note as its title; shell fold times join the subagent fold timer.
- `src/model.ts`: full `Shell` type from `state.rs`. `src/rowView.ts`: `shellNote`. `src/strip.ts`/`stripPlate.ts`: the strip cell's hover adds the same note.
- `src/demo.ts`: shells on ingest-01 (serving :5173, a long running command), digest-04 (an exit that folds about 27s after load, one already folded) and sync-05 (quiet, and a failed build that folds about 18s in while its dot stays). No Rust change.

## Decisions taken without a ruling

- Chip layout: one square line per shell on the subagent rail (2px border in `--ps-line`, panel-2 fill): `>_` glyph in the state colour, command, runtime, outlined state label. Colours: running = working cyan, quiet = stuck grey, serving = success green, exited = dim, failed = error red (failed also gets a red border).
- Labels: `Running`, `Quiet`, `Serving :<port>` (`Serving` with no port), `Exited`, `Failed <code>` (`Failed` with none). Exit code, description and last line go in the chip's hover, not on the line: at 354px the command already gets only ~16 characters.
- Middle truncation is by character count from the measured width (IBM Plex Mono is 0.6em per glyph), so both ends of a command stay. The full command is the chip's title.
- Runtime runs from `startedAt` to now while live and freezes at `endedAt`.
- `failed` chips fold after 30s too, like `exited`. The parent's red dot is what survives: it stays while the backend holds any failed shell for that session (up to 20 finished, AV-29), on the closed row, the open stage and the strip cell. There is no dismiss. "Collapsed" is read as the row closed, the chip folded, or the widget in strip mode; the dot shows in all three.
- The dot sits on the parent's peep (top-left, 10px, red, ringed in panel colour), mirroring the strip's existing `.ps-strip-fail`. Its title lists every failed shell, newest first: command, exit code (or "no exit code"), last line.
- Folded chips sit behind a `>_ ended N` button (same toggle style as `done N`) that shows them newest first. No cap on live chips (subagents' 4 + "+N more" not applied): sessions rarely hold many shells.
- Order: live oldest-started first, then lingering ended newest first.

## Live check 2026-11-15

`npm run tauri dev`, plus the browser demo (`npm run dev`, `?demo`) in headless Edge. Screenshots in the run's scratchpad `AV-30\`.

- **Real shells from this agent's own session** (hooks already installed): a `node` HTTP server on :47955 showed `SERVING :47955` with a ticking runtime within one probe tick (`live2-0.png`). The real failing (`exit 3`) and clean commands stayed `RUNNING`: see Owed.
- **Synthetic hooks** on placeholder `av30-live` (PostToolUse Bash `backgroundTaskId` x4, then two `<task-notification>` prompts): `FAILED 2` with a red border, `EXITED`, two `RUNNING`, and the red dot on the parent (`live3-0.png`). 32s later both ended chips had folded into `>_ ended 2`, the dot was still there and the window shrank (`live4-folded-0.png`). `Ctrl+Alt+P` to the strip: the av30-live cell carried the red flag (`live5-strip-0.png`).
- **Hover text** checked in the demo DOM, not as an on-screen tooltip: the dot's title is `Shell failed: npm run build / exit code 1 / last: error TS2307: …`, the chip's title adds the description, and the strip cell's title appends the same note.
- Demo dark, light and after-33s shots (`demo-dark.png`, `demo-light.png`, `demo-after-30s.png`): chips readable in both themes, no overlap; the digest exit and the sync build folded on time with sync-05's dot intact.

## Done 2026-11-15

Owner closed it 2026-11-15 on the build installed 19:44. What was owed, as it stood:

- The owner has not seen it. Look at a real dev server chip with its port, and a real failed build on a closed row and in the strip. Hover the red dot and the strip cell to confirm the tooltip shows exit code and last line.
- Middle truncation is tight at 354px (`npm run …rt 5173`): the owner may want the runtime or label shorter, or a two-line chip.
- **Backend finding (AV-29, not fixed here):** shells started from a subagent-run session (this worker) never ended. Their `<task-notification>` went to the agent, not through a `UserPromptSubmit` hook. Their output files were under a different session folder (`…\claude\<hash>\<parent session id>\tasks\`) from the hook's `session_id`, so the output-file trailer fallback never found them. They stayed `RUNNING` until the widget was stopped. A shell started from an interactive top-level session (AV-29's live check) is not affected.
- Not seen: `quiet` live (needs 2 min of silence on a real shell), and the strip hover tooltip on screen.
