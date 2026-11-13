# AgentView spec

Status: draft 2026-11-13. Decisions come from the design grill recorded in `CLAUDE.md`. Items marked **Verify** are assumptions about Claude Code internals that the ticket implementing them must confirm against a live session before building on them.

## 1. Goal

A glanceable Windows desktop widget that answers three questions without switching windows:

1. Which Claude Code sessions are running, and what is each doing right now?
2. Does any of them need me?
3. How full is each context window, and how close am I to my rate limits?

## 2. Non-goals (v1)

- Agents other than Claude Code (the adapter seam exists, no second adapter ships).
- Acting on sessions: no replying, approving permissions, killing, or starting sessions.
- Token counts, cost, history of ended sessions.
- macOS/Linux. Code should not gratuitously block them, but nothing is tested there.
- Changing the peep art. `design/peep.js` is the user's; new poses are a request to the user.

## 3. User-visible behaviour

### 3.1 Window

- Frameless, always on top, no taskbar entry. Docks to the right edge of the primary monitor on first run, about 360px wide, height fits its rows up to a cap, then scrolls.
- Draggable by its header. Position and monitor persist across restarts; if the saved monitor is gone, fall back to the default dock.
- Tray icon: left click toggles the widget, menu has Show/Hide, Install integration / Uninstall integration, Start at login (checked by default), Quit.
- Starts at login.

### 3.2 Header

App name and account-wide rate limits: `5h 42%  ·  7d 18%`, each with a thin bar. Values come from the most recent statusline payload of any session, with its age shown on hover. Hidden until the first payload arrives.

### 3.3 Session row (compact)

Follows the "In a monitor" mock in `design/peep-this.dc.html`, scaled to the widget:

| Column | Content |
|---|---|
| Peep | `<peep-bot>` at 32px, state from §4, colour and accessory from §3.5 |
| Name | Session name from the session file (`name`), IBM Plex Mono, ellipsised |
| Task | Per-state text (§4) — the last tool for `working`, else the state's canned line |
| State | LED dot and label in the state's LED colour |
| Context | Thin bar under the row, fill = used context %. Absent until a statusline payload arrives |
| Badge | Subagent count on the peep's corner when > 0 |

Sort: `waiting` first, then by most recent state change.

### 3.4 Session row (expanded)

Click toggles. Adds a 64px peep plus: cwd, model, last tool with its target (file or command, truncated), time in current state, context `used / max` %, session kind and entrypoint. Only one row expanded at a time.

### 3.5 Identity

Stable FNV-1a hash of the normalised cwd (lower-cased, forward slashes, no trailing slash) picks one of 5 colours (yellow, pink, lavender, blue, green) and, from different bits, one of 7 accessories (none, hardhat, headphones, glasses, bow, party, chef). Two sessions in the same project look identical; the name tells them apart.

### 3.6 Alerts

When a session enters `waiting`:

- Windows toast: title = session name, body = the notification message from the hook (e.g. "Claude needs your permission to use Bash"). Clicking it focuses the session's terminal.
- The row pulses (subtle amber glow, about 1.5s period) until the state leaves `waiting`.
- No toast if the widget's own window is focused and the row is visible, or if the same session toasted within the last 30s.

Clicking a row's peep or name focuses the session's terminal (§6.6).

### 3.7 Ended sessions

When the session's process exits, the row shows `success` if the last turn ended cleanly, else `idle`, dims, and fades out after 60s.

## 4. State model

Seven states from peep, derived per session by the backend. Peep's `energy` stays at the default 3.

| State | Entered on | Task text |
|---|---|---|
| `idle` | Session discovered with nothing in flight; `SessionStart` | "Idle" |
| `thinking` | `UserPromptSubmit`; `PostToolUse` (model is generating between tools) | "Thinking" |
| `working` | `PreToolUse` | `<Tool> <target>` e.g. `Edit src/main.ts` |
| `waiting` | `Notification` (permission prompt, question, or idle prompt) | Notification message |
| `success` | `Stop` | "Done" — held until the next `UserPromptSubmit` |
| `error` | Turn ended by an API error (**Verify** the signal: `StopFailure` hook if present, else transcript `isApiErrorMessage`) | Error summary |
| `stuck` | In `thinking` or `working` with no event for 6 min | "No response for N min" |

Rules:

- Any event resets the stuck timer. `stuck` leaves on the next event.
- `waiting` leaves on the next `PreToolUse`, `PostToolUse`, `UserPromptSubmit` or `Stop`. **Verify** which event follows an approved permission prompt.
- A `Notification` whose type is the idle reminder (Claude waiting for a new prompt after finishing) does not override `success`; it maps to `success` with no toast. **Verify** the field that distinguishes notification kinds.
- Without hooks (session predates install, or integration not installed): `busy` in the session file maps to `working`, `idle` to `idle`, with transcript tailing (§6.4) refining it when available.
- Subagents: `PreToolUse` for the `Task`/`Agent` tool increments the count; `SubagentStop` decrements; clamp at 0; reset on `Stop`. **Verify** tool name.

The derivation is a pure function `(SessionState, Event, now) -> SessionState` in Rust, fully unit-tested from fixtures. Timers call the same function with a synthetic `Tick` event.

## 5. Architecture

```
 Claude Code sessions
   │  ~/.claude/sessions/<pid>.json ──────────────┐ (watch)
   │  hooks ──► agentview-hook.exe ──HTTP──┐      │
   │  statusLine ──► agentview-statusline ─┤      │
   │  ~/.claude/projects/**/<id>.jsonl ────┼──────┤ (tail, fallback)
                                           ▼      ▼
                              ┌──── Rust backend (Tauri) ────┐
                              │ ingest ─► state engine ─► store
                              │ timers ─┘                │   │
                              │ notifier, focus, tray ◄──┘   │
                              └──────────────┬───────────────┘
                                             │ Tauri event "sessions"
                                             ▼
                              TS frontend: rows + <peep-bot>
```

### 5.1 Backend modules (`src-tauri/src/`)

| Module | Responsibility |
|---|---|
| `source/` | `AgentSource` trait (`discover`, `events`) and the `claude` implementation. The seam for future Codex/Gemini adapters. Nothing outside `source/` knows Claude Code file formats. |
| `source/claude/sessions.rs` | Parse and watch `~/.claude/sessions/*.json`; liveness by PID + `procStart` (guards PID reuse). |
| `source/claude/transcript.rs` | Tail a session's transcript JSONL from its last offset; derive model, last tool, API errors. |
| `ingest/server.rs` | `127.0.0.1` HTTP listener for hook and statusline payloads; token check. |
| `state.rs` | `Session`, `State`, `Event`, the pure transition function, stuck timer. |
| `store.rs` | Map `sessionId -> Session`; emits a full snapshot to the frontend on change, throttled to at most every 100ms. |
| `notify.rs` | Toasts, de-duplication rules (§3.6). |
| `focus.rs` | PID to top-level window resolution and foregrounding (§6.6). |
| `window.rs`, `tray.rs` | Docking, persistence, tray menu, autostart. |
| `install.rs` | Settings merge and statusline wrap, with backup and uninstall (§7). |

### 5.2 Frontend (`src/`)

Thin. Listens for the `sessions` snapshot, renders rows, handles expand/collapse and click-to-focus (`invoke("focus_session", { sessionId })`). Contains `vendor/peep.js` unchanged, the identity hash (§3.5), and formatting. Vitest covers identity hashing, sort order and formatting.

### 5.3 Snapshot shape (backend to frontend)

```ts
type Snapshot = {
  sessions: Session[];
  rateLimits?: { fiveHourPct: number; sevenDayPct: number; at: number };
  integration: "installed" | "missing" | "outdated";
};

type Session = {
  id: string;            // sessionId
  pid: number;
  name: string;
  cwd: string;
  kind: string;          // e.g. "interactive"
  entrypoint: string;    // e.g. "cli"
  model?: string;
  state: "idle" | "thinking" | "working" | "waiting" | "success" | "error" | "stuck";
  stateSince: number;    // epoch ms
  task: string;
  lastTool?: { name: string; target?: string };
  subagents: number;
  contextPct?: number;   // used, 0–100
  alive: boolean;
  endedAt?: number;
};
```

## 6. Data sources

### 6.1 Session files

`~/.claude/sessions/<pid>.json`, observed fields: `pid`, `sessionId`, `cwd`, `startedAt`, `procStart`, `version`, `kind`, `entrypoint`, `name`, `status` (`busy`/`idle`), `updatedAt`, `statusUpdatedAt`. Parse leniently: unknown fields ignored, missing optional fields tolerated, a malformed file skipped and logged, never fatal. A file whose PID is dead (or whose `procStart` mismatches) is an ended session.

### 6.2 Hooks

Installed hooks run `agentview-hook.exe <EventName>`, which reads the hook JSON from stdin, POSTs it to the widget, and exits 0 within 300ms whether or not the widget is running. It must never block or fail a Claude Code turn and never print to stdout (some hook events treat stdout as context). Events: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Notification`, `Stop`, `SubagentStop`, `SessionEnd`, plus `StopFailure` if it exists.

**Verify** whether Claude Code supports native HTTP hooks (`"type": "http"`); if so prefer them over the forwarder binary and drop `agentview-hook.exe`.

### 6.3 Statusline

`agentview-statusline.exe` wraps the user's original statusline command. It reads stdin, runs the original command with the same stdin, prints the original's stdout unchanged, then POSTs the payload in a detached, fire-and-forget way. The rendered statusline must be identical to before install and add no perceptible delay (budget: under 50ms overhead). Used fields: `session_id`, `model.display_name`, `context_window.remaining_percentage` (context used = 100 − remaining), `rate_limits.five_hour.used_percentage`, `rate_limits.seven_day.used_percentage`.

### 6.4 Transcripts (fallback)

`~/.claude/projects/<slug>/<sessionId>.jsonl`. Tail only sessions that have not produced a hook event in the last 10s, from the stored byte offset; never re-read whole files after the first scan, and cap the first scan to the last 256KB. Used to fill model, last tool, and API errors, and to infer `working` vs `thinking` for sessions without hooks.

### 6.5 Event server

- Binds `127.0.0.1` only, on a fixed default port (47821) with fallback to a random free port. The chosen port and a random 32-byte token are written to `%APPDATA%\AgentView\endpoint.json` on startup; the hook and statusline binaries read it per invocation.
- `POST /hook/<EventName>` and `POST /statusline`, header `X-AgentView-Token`. Wrong or missing token: 401, payload dropped.
- Body cap 1MB. Payloads are parsed into the fields listed above and the rest discarded; prompts, tool inputs and outputs are not stored beyond the derived `task` string.

### 6.6 Focusing a session

Walk the process tree from the session PID up to the first ancestor that owns a visible top-level window (Windows Terminal, conhost, VS Code, Cursor, JetBrains, the Claude desktop app), restore it if minimised, and bring it to the foreground. Windows Terminal tabs cannot be targeted individually: the WT window is focused, not the tab. Documented limitation. If no window is found, the widget flashes the row instead.

## 7. Integration install

Triggered from the tray or a first-run banner. Never automatic.

1. Copy `~/.claude/settings.json` to `~/.claude/settings.json.agentview-backup-<timestamp>`.
2. Merge hook entries for each event in §6.2. Existing user hooks are kept; AgentView's entries are identified by their command path so reinstall is idempotent and uninstall removes only them.
3. If `statusLine` exists, record its original command in `%APPDATA%\AgentView\statusline-original.json` and replace it with `agentview-statusline.exe`. If none exists, install the wrapper with no original (it then prints nothing).
4. Write with a temp file and atomic rename. Validate the result parses as JSON before replacing.
5. Uninstall reverses 2 and 3 exactly, restoring the recorded statusline command.
6. `integration: "outdated"` when installed entries point at a different binary path or version than the running app.

## 8. Performance and robustness budgets

- Idle: under 1% CPU, under 80MB RAM with 10 sessions.
- Peep animation runs only for visible rows (peep's own IntersectionObserver) and pauses entirely while the widget is hidden.
- File watching uses OS notifications, with a 5s poll as a safety net.
- Any malformed input (session file, hook body, transcript line) is skipped and logged, never fatal.
- Logs to `%APPDATA%\AgentView\logs`, rotated, no prompt or tool content.

## 9. Testing

- `cargo test`: session file parsing (fixtures from real files, anonymised), state transitions (one fixture sequence per §4 row and rule), install merge and uninstall round trip on fixture settings files, endpoint token check.
- Vitest: identity hashing stability, sort order, formatting.
- Fixtures under `src-tauri/tests/fixtures/` and `src/**/__fixtures__/`.
- Manual check per surface ticket: run `npm run tauri dev` against live sessions and confirm what the owner sees.

## 10. Open questions

None blocking. The **Verify** items above are resolved inside their tickets and recorded there.
