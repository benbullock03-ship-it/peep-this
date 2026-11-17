# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

Tauri 2 + Vite + TS; core backend (state engine, session files, event server, transcripts) in place, UI surfaces in progress. Full behaviour is in `docs/spec.md`; work is in `tickets/` (`AV-NN-*.md`, frontmatter `status`/`depends`/`attended`). Ticket statuses are `open`, `doing`, `blocked`, `deferred`, `done`, `closed`: `done` is Claude's claim, `closed` the owner's confirmation and the only status that moves a ticket to `tickets/archive/` (`git mv`); `npm run tickets` indexes both folders. The decisions below come from the design grills on 2026-11-13 and 2026-11-14 (mission control, then surface) and the domain-modelling session on 2026-11-15 (`CONTEXT.md`, `docs/adr/`), and stand until the user overrides them.

## Commands

Run from the repo root (Git Bash or PowerShell).

| Task | Command |
|---|---|
| Run the widget (dev, hot reload) | `npm run tauri dev` |
| Frontend only in a browser | `npm run dev` (port 1420; Tauri APIs unavailable) |
| Type-check + build frontend (fast tier) | `npm run build` |
| Frontend tests | `npm test`; one file: `npx vitest run src/path/file.test.ts`; by name: `npx vitest run -t "name"` |
| Rust check / tests | `cargo check` / `cargo test` in `src-tauri/`; one test: `cargo test <name>` |
| Full run | `npm run build && npm test && (cd src-tauri && cargo test)` |
| Regenerate ticket index | `npm run tickets` (writes `tickets/INDEX.md`) |
| Release build | `npm run tauri build`. Never publish without an explicit yes |

First `cargo` build takes ~5 min; later ones are incremental.

## What this is

Peep This: an always-on-top desktop companion for new Claude Code users, the widget and the `peep` skill pack installed as one (ADR 0001). The widget is a Windows window that lists the Claude Code sessions running on this machine. Each row has a peep, a pixel marshmallow that shows the session's state. The widget alerts the user when a session needs input and shows context-window and rate-limit usage. The skill pack teaches a working rhythm. `CONTEXT.md` is the glossary; use its terms.

## Design (`design/`)

- `peep.js` is the user's art and the **source of truth for the creature**. It is a dependency-free `<peep-bot>` custom element drawn on a 32×32 canvas. `src/vendor/peep.js` and `plugin/skills/goose/assets/peep.js` are byte-identical copies; if the user sends a new version, replace all three. Do not redraw, restyle or add poses; any change to the art goes back to the user.
  - API: attributes `state`, `color` (palette name or any CSS colour), `accessory`, `energy` (1–10, sets 8–12 fps), `size`, `hover="off"`. JS setters: `el.state = 'waiting'`.
  - One shared rAF loop. Rendering pauses when a peep is off-screen (IntersectionObserver).
- `peep-this.dc.html` is the Claude Design reference page. It runs on `support.js`, the Claude Design runtime, which is not app code. Its "In a monitor" section is the row mock.
- `peep-this-preview.png` is a screenshot of the reference page.
- Visual language: dark background `#141218`, panels `#1A1820`, borders `rgba(255,255,255,0.07)`, text `#EDE9F2` / `#A49FB0` / `#7D7889`, accent `#F7D64A`. Fonts are Fredoka (display), IBM Plex Sans (UI) and IBM Plex Mono (session names). Bundle the fonts locally; the widget must work offline.

## Decided

- **Scope:** Claude Code sessions only (CLI, desktop, IDE). Put agent sources behind an adapter interface so Codex/Gemini can be added later without touching the UI.
- **Stack:** Tauri 2. Rust backend for file watching, the localhost hook server, window focus and tray. TypeScript web frontend that hosts `<peep-bot>`.
- **Form:** frameless always-on-top panel, autostarts at login, docks to the right edge of the primary monitor, remembers a dragged position. Tray icon shows/hides it.
- **States:** peep's seven, derived per session:

  | State | Trigger |
  |---|---|
  | `thinking` | `UserPromptSubmit`, or `PostToolUse` (model generating between tools) |
  | `working` | `PreToolUse` until the matching `PostToolUse` |
  | `waiting` | `Notification` (permission prompt or question) |
  | `success` | `Stop`; held until the next prompt |
  | `error` | API/tool failure |
  | `stuck` | busy with no event for 6 min |
  | `idle` | session open, nothing in flight |

  Running subagents show as a small count badge on the row, not as a state.
- **Identity:** the project cwd hashes to one of the 5 palette colours (yellow, pink, lavender, blue, green) and one of the 14 accessories, so a project always looks the same.
- **Shell (owner design 2026-11-14, AV-44, `design/shell/`):** "Pixel desktop": square card, 2px border, 6px hard shadow, striped title bar, square speech bubbles, peeps on striped shelf planks. `design/shell/` is the reference, like `peep.js` is for the creature; adapt it, don't restyle it.
- **Size:** 48px peep in rows, 96px on the open row's stage, 40px in the strip, 32px for subagent children. 48 and 40 are the owner's ruling over the old multiples-of-32 rule; don't introduce other sizes.
- **Rows:** peep on a plank, a speech bubble saying what the session is doing, session name (mono) and an outlined state chip. Attention states (`waiting`, `error`, `stuck`) fill the bubble with the state colour and sort first. Click opens the row in place: stage, tags (time in state, model, kind), last tool, context bar, cwd, decision trail.
- **Alerts:** on entering `waiting`, show a Windows toast and pulse the row. Clicking either focuses that session's terminal window.
- **Actions:** focus, permission decisions (see Mission control) and Jump: copy a command (a floppy's `/peep:<skill>`, `/peep:ticket <id>` from the ticket shelf, `/peep:packup`) and focus that session's terminal for the user to paste. End session: a 1s hold on the open row only (never strip, flyout or toast) terminates the session's PID, refused if `procStart` no longer matches (ADR 0003). It never types into a session, answers questions, sends replies or starts sessions.
- **Usage shown:** per-session context % and account 5-hour/7-day rate-limit %. No token counts or cost.
- **Lifecycle:** when a session's PID dies, show a done state, then fade it out after about 1 minute.
- **Churn rule (AV-131):** a new skill or widget feature needs a named user problem and one real use before it gets art.
- **v1 cut line (AV-131):** rows, waiting alerts, inline approvals, bars, the signpost (AV-129), subagent and shell rows, the six Core floppies and Big Peep. Everything else comes after the release. Coderot stays in Extras.
- **9/10 bar (AV-131):** a newcomer installs in one step, gets a signpost Jump within 5 minutes without reading docs, runs `/peep:ticket` and `/peep:packup` unprompted by day 2, and nothing breaks across one Claude Code release.
- **Release (AV-131):** v1 ships as an unwatched open beta through the Claude Code plugin marketplace. Testers prove the bar by pasting their rhythm counter into a beta issue. macOS comes straight after v1.

## Mission control (grill 2026-11-14, spec §11, tickets AV-20 to AV-33)

Planned, not built. This overturns the earlier "read-only, focus only" rule. Build order: approvals, then subagents and shells, then rules, then digest.

- **Thesis:** the widget is where the fleet is run from; the terminal is the detail view.
- **Inline approvals:** allow, deny, deny with reason, by answering the `PermissionRequest` hook. Widget holds the request about 60s, then releases to the terminal prompt; releases at once if that terminal is focused. Decisions come only from the widget UI, never over HTTP.
- **Risk tiers:** `safe` (reads, in-project edits) one click; `shell` one click with the full command; `destructive` hold 1s, never from a toast, never auto-allowed. Classifier fails closed.
- **Surfaces:** approval card under the row with a real diff or full command; global-hotkey keyboard inbox (`Y`/`N`/`R`/`Tab`); toast buttons for `safe` only.
- **Trail:** every decision logged locally and shown on the expanded row.
- **Subagents:** indented child rows, each a 32px peep in the parent's colour with an accessory hashed from the agent type. Finished ones linger 30s with their result, then fold into a "done N" counter. Replaces the count badge.
- **Shells:** chip rows under the parent with five states: running, quiet, serving (port shown), exited, failed. Failed bubbles a red dot to the parent.
- **Rules:** suggested after three identical-shaped allows, confirmed by the user, stored in the widget, revocable from the trail.
- **Digest:** while-you-were-away card from events; model summary via `claude -p` is opt-in.
- **Not taken:** collision radar (except clash as an ambient mood, ADR 0002), forecasts, phone remote, trust dial, global pause, orphan shells, launching sessions.

## Skill pack (domain modelling 2026-11-15, `CONTEXT.md`, ADRs 0001 and 0002, spec §13, tickets AV-53 to AV-71)

Planned, not built. `CONTEXT.md` is the glossary and wins over older wording here or in the spec. Build order: pack (AV-55 to AV-62), widget (AV-64 to AV-68), moods (AV-69 to AV-71); AV-54 verifies hook payloads before the widget phase.

- **Pack:** one Claude Code plugin `peep` in `plugin/`, installed with the integration: `/peep:ticket`, `/peep:ducks`, `/peep:qt` (Question Time), `/peep:packup` (Pack Up), `/peep:shoot`, `/peep:coderot`. Every Peep skill reads the repo setup instead of guessing.
- **Widget:** a floppy per Peep skill; inserted floppy on the plank while it runs; floppy tray and ticket shelf on the open row, each item a Jump. `design/skill-pack/` is the reference, like `design/shell/`.
- **Moods:** at most one per peep, shown as an emote in the bubble; never changes the state; an attention state takes the bubble first. Clash is a mood: no warning, no blocking, no trail entry.

## Surface (surface grill 2026-11-14, spec §12, tickets AV-34 to AV-43)

Planned, not built. Build order: verify, chrome, bars and motion, snap, strip, then hotkey, peek, flyout, empty state.

- **Chrome:** transparent window; square card per AV-44 (2px border, hard black shadow offset 6px, no blur). No edge light. The three-stair corners were overruled 2026-11-14.
- **Bars:** context and rate limits are 10 square blocks; amber from 70%, red from 90%.
- **Motion:** stepped, 4 to 6 frames over about 160ms. Reduced motion is instant.
- **Snap:** live magnet within 16px of work-area edges and corners on any monitor; floats elsewhere. The snapped edge is the anchor for every resize.
- **Modes:** `panel`, `strip`, `hidden`; launches in `strip`. `Ctrl+Alt+P` toggles panel/strip, tray click toggles hidden. No `Esc`.
- **Strip:** AV-44-design plate of 40px peeps, parents only, 8 then `+N`; column on side edges, row on top/bottom. Hover 200ms peeks, click pins.
- **Waiting while collapsed:** only that row flies out from its peep; the approval card mounts there.
- **Empty:** one house peep, `idle`, yellow, no accessory.
- **Not taken:** pill, edge tab, Mica, perch card, floating chips, coloured shadow, ghost outline, auto-collapse, auto-expand.

## Data sources (hybrid)

| Signal | Source |
|---|---|
| Session list, busy/idle, name, cwd, version | `~/.claude/sessions/<pid>.json` (watch the directory; one file per live session, removed or stale when the PID exits) |
| State transitions | Claude Code hooks (`UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Notification`, `Stop`, `SubagentStop`) POST to the widget's localhost port |
| Context %, rate limits | The statusline payload (`context_window.remaining_percentage`, `rate_limits.five_hour/seven_day.used_percentage`), forwarded by a wrapper around the user's existing statusline script |
| Fallback | Tail `~/.claude/projects/<slug>/<sessionId>.jsonl` transcripts |

Join hook and statusline events to sessions by `sessionId`. Sessions that started before the hooks were installed still show up through the sessions directory.

### Integration install

The widget offers a one-click install that **backs up `~/.claude/settings.json` first**, merges the hook entries, and wraps the existing `statusLine` command. The wrapper must keep printing the user's original statusline output unchanged and only tee the JSON to the widget. Uninstall reverses both changes. Never overwrite user hooks or the statusline.

## Architecture in one breath

The Rust backend owns all truth: it watches session files, receives hook and statusline POSTs on a loopback port, tails transcripts as fallback, runs every input through one pure state function (`state.rs`, spec §4), and pushes a full snapshot to the frontend as a `sessions` Tauri event. The TS frontend only renders that snapshot and calls `focus_session`. Claude Code file formats stay inside `src-tauri/src/source/claude/` behind the `AgentSource` trait. Module map: spec §5.1.

## Testing

Vitest for the frontend (identity hash, sort, formatting); `cargo test` for parsers, the state engine and installer round trips. Fixtures are anonymised copies of real session files, hook payloads and transcript lines (`src-tauri/tests/fixtures/`). No E2E. Surface tickets also need the owner to confirm what they saw in `npm run tauri dev`.
