# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

Scaffolded (Tauri 2 + Vite + TS), no features yet. Full behaviour is in `docs/spec.md`; work is in `tickets/` (`AV-NN-*.md`, frontmatter `status`/`depends`/`attended`). The decisions below come from the design grill on 2026-11-13 and stand until the user overrides them.

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

AgentView: an always-on-top Windows desktop widget that lists the Claude Code sessions running on this machine. Each row has a "peep", a pixel marshmallow desktop pet that shows the session's state. The widget alerts the user when a session needs input and shows context-window and rate-limit usage.

## Design (`design/`)

- `peep.js` is the user's art and the **source of truth for the creature**. It is a dependency-free `<peep-bot>` custom element drawn on a 32×32 canvas. `src/vendor/peep.js` is a byte-identical copy; if the user sends a new version, replace both. Do not redraw, restyle or add poses; any change to the art goes back to the user.
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
- **Identity:** the project cwd hashes to one of the 5 palette colours (yellow, pink, lavender, blue, green) and one of the 7 accessories, so a project always looks the same.
- **Size:** 32px peep in compact rows, 64px when a row is expanded. Use multiples of 32 only; other sizes render unevenly sized pixels.
- **Rows:** the compact row follows the mock: peep, session name (mono), task text, state LED and label, plus a thin context bar. Click to expand cwd, model, last tool and time in state.
- **Alerts:** on entering `waiting`, show a Windows toast and pulse the row. Clicking either focuses that session's terminal window.
- **Actions:** focus only. The widget is read-only. It never answers prompts, approves permissions or kills sessions.
- **Usage shown:** per-session context % and account 5-hour/7-day rate-limit %. No token counts or cost.
- **Lifecycle:** when a session's PID dies, show a done state, then fade it out after about 1 minute.

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
