# Peep This spec

Status: draft 2026-11-13, revised 2026-11-14. Decisions come from the design grills recorded in `CLAUDE.md`. §1 to §10 describe what is built; §11 is the mission-control direction from the second grill and §12 the surface direction from the third, both planned and not yet built. Items marked **Verify** are assumptions about Claude Code internals that the ticket implementing them must confirm against a live session before building on them.

## 1. Goal

A glanceable Windows desktop widget that answers three questions without switching windows:

1. Which Claude Code sessions are running, and what is each doing right now?
2. Does any of them need me?
3. How full is each context window, and how close am I to my rate limits?

From 2026-11-14 a fourth: can I deal with what needs me from here, without opening the terminal? (§11)

## 2. Non-goals (v1)

- Agents other than Claude Code (the adapter seam exists, no second adapter ships).
- Acting on sessions beyond permission decisions (§11.1): no replying, answering questions, killing, or starting sessions.
- Token counts, cost, history of ended sessions.
- macOS/Linux. Code should not gratuitously block them, but nothing is tested there.
- Changing the peep art. `design/peep.js` is the user's; new poses are a request to the user.

## 3. User-visible behaviour

### 3.1 Window

- Frameless, always on top, no taskbar entry. Docks to the right edge of the primary monitor on first run, about 360px wide, height fits its rows up to a cap, then scrolls.
- Draggable by its header. Position and monitor persist across restarts; if the saved monitor is gone, fall back to the default dock.
- Transparent window (AV-35). The panel is the AV-44 card: square, 2px border in the text colour, hard shadow `rgba(0,0,0,0.45)` offset 6px right and down, no blur. Border and shadow never carry state. The window is larger than the card by the shadow, and docking, clamping and height fit use the card rect, so the shadow may overhang the 12px margin. A window region limits hit testing to card plus shadow, so clicks on the transparent corners reach the window underneath.
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

Stable FNV-1a hash of the normalised cwd (lower-cased, forward slashes, no trailing slash) picks one of 5 colours (yellow, pink, lavender, blue, green) and, from different bits, one of 14 accessories (none, hardhat, headphones, glasses, bow, party, chef, crybaby, beanie, crown, shades, flowers, halo, wizard). Two sessions in the same project look identical; the name tells them apart.

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
| `waiting` | `PermissionRequest`; `Notification` (permission prompt, question) | "Needs permission: `<Tool> <target>`" or the notification message |
| `success` | `Stop` | "Done" — held until the next `UserPromptSubmit` |
| `error` | Turn ended by an API error: `StopFailure` hook, else transcript `isApiErrorMessage` | "API error: `<error_details or error type>`" |
| `stuck` | In `thinking` or `working` with no event for 6 min | "No response for N min" |

Rules:

- Any event resets the stuck timer. `stuck` leaves on the next event.
- `waiting` leaves on the next `PreToolUse`, `PostToolUse`, `UserPromptSubmit` or `Stop`. Order is `PreToolUse`, then `PermissionRequest` while the prompt shows, then `PostToolUse` once the tool has run after approval.
- A `Notification` with `notification_type: "idle_prompt"` (Claude waiting for a new prompt after finishing) does not override `success`; it maps to `success` with no toast.
- Without hooks (session predates install, or integration not installed): `busy` in the session file maps to `working`, `idle` to `idle`, with transcript tailing (§6.4) refining it when available.
- Subagents: `PreToolUse` for the `Task`/`Agent` tool increments the count; `SubagentStop` decrements; clamp at 0; reset on `Stop`. The tool is `Agent` (`Task` is still accepted). The count stays until AV-28.
- Subagent entries (AV-27): events with `agent_id` set drive that agent's own `Agent` entry by the same rules and never the parent's task or tool; they reset the parent's stuck timer. `SubagentStart` creates the entry (fallback: the first event with a new `agent_id`), taking its description from the oldest queued `PreToolUse Agent`. `SubagentStop` ends it with a result line: the first line of `last_assistant_message`, or a hand-back message. A subagent's permission request puts the parent in `waiting` until that agent's next event. `Stop` does not end agents, so background agents outlive the turn. Nested agents are listed flat. Up to 20 finished agents are kept.
- Background shells (AV-29): a Bash/PowerShell `PostToolUse` with `tool_response.backgroundTaskId` starts a `running` shell. A `<task-notification>` prompt ends it with its status and exit code (and still counts as `UserPromptSubmit`). `TaskStop` ends it as `exited` with no code. `Stop.background_tasks` only adopts shells the widget missed, never ends one. A 5s probe outside the pure function (`ticker.rs`, `source/claude/shells.rs`) finds the shell's wrapper under the session PID, the lowest listening port in its tree (`serving`) and the output file's last write (`quiet` after 2 min), last line and exit marker. Probe events never reset the stuck timer. Shells end with the session; 20 finished are kept.

The derivation is a pure function `(SessionState, Event, now) -> SessionState` in Rust, fully unit-tested from fixtures. Timers call the same function with a synthetic `Tick` event.

## 5. Architecture

```
 Claude Code sessions
   │  ~/.claude/sessions/<pid>.json ──────────────┐ (watch)
   │  hooks (native http) ─────────HTTP────┐      │
   │  statusLine ──► peepthis-statusline ──┤      │
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
| `ingest/hooks.rs` | The native HTTP hook entries the installer merges into settings.json. |
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
  agents: { id: string; agentType: string; description: string; state: string; stateSince: number; task: string;
            lastTool?: { name: string; target?: string }; startedAt: number; endedAt?: number; toolCount: number; result?: string }[];
  shells: { id: string; command: string; description: string; state: "running" | "quiet" | "serving" | "exited" | "failed";
            stateSince: number; startedAt: number; endedAt?: number; port?: number; exitCode?: number; lastLine?: string }[];
  contextPct?: number;   // used, 0–100
  alive: boolean;
  endedAt?: number;
};
```

## 6. Data sources

### 6.1 Session files

`~/.claude/sessions/<pid>.json`, observed fields: `pid`, `sessionId`, `cwd`, `startedAt`, `procStart`, `version`, `kind`, `entrypoint`, `name`, `status` (`busy`/`idle`), `updatedAt`, `statusUpdatedAt`. Parse leniently: unknown fields ignored, missing optional fields tolerated, a malformed file skipped and logged, never fatal. A file whose PID is dead (or whose `procStart` mismatches) is an ended session.

### 6.2 Hooks

Native Claude Code HTTP hooks (`"type": "http"`), verified in AV-06; there is no forwarder binary. Each entry POSTs the hook JSON to `http://127.0.0.1:<port>/hook/<EventName>` with the token as a literal `X-PeepThis-Token` header and `"timeout": 1`. Built by `ingest/hooks.rs`. An unreachable widget is a non-blocking hook error and never fails a turn. The server answers 204 with an empty body, which Claude Code treats as no decision (this matters for `PermissionRequest`).

Events: `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`, `Notification`, `Stop`, `StopFailure`, `SubagentStart`, `SubagentStop`, `SessionEnd`. `SessionStart` does not accept `http` hooks; the sessions directory covers it. Shell lifecycle rides on existing events: `backgroundTaskId` in `PostToolUse`, `<task-notification>` prompts in `UserPromptSubmit`, and `background_tasks` in `Stop`.

The token persists in `endpoint.json` across restarts, because settings.json holds it literally. The URL holds the port, so hooks reach the widget only on the default port 47821.

### 6.3 Statusline

`peepthis-statusline.exe` wraps the user's original statusline command. It reads stdin, runs the original command with the same stdin, prints the original's stdout unchanged, then POSTs the payload in a detached, fire-and-forget way. The rendered statusline must be identical to before install and add no perceptible delay (budget: under 50ms overhead). Used fields: `session_id`, `model.display_name`, `context_window.remaining_percentage` (context used = 100 − remaining), `rate_limits.five_hour.used_percentage`, `rate_limits.seven_day.used_percentage`.

### 6.4 Transcripts (fallback)

`~/.claude/projects/<slug>/<sessionId>.jsonl`. Tail only sessions that have not produced a hook event in the last 10s, from the stored byte offset; never re-read whole files after the first scan, and cap the first scan to the last 256KB. Used to fill model, last tool, and API errors, and to infer `working` vs `thinking` for sessions without hooks.

### 6.5 Event server

- Binds `127.0.0.1` only, on a fixed default port (47821) with fallback to a random free port. The chosen port and a random 32-byte token are written to `%APPDATA%\PeepThis\endpoint.json` on startup. The token is generated once and then reused from that file, since installed hooks hold it. The statusline binary reads the file per invocation.
- `POST /hook/<EventName>` and `POST /statusline`, header `X-PeepThis-Token`. Wrong or missing token: 401, payload dropped.
- Body cap 1MB. Payloads are parsed into the fields listed above and the rest discarded; prompts, tool inputs and outputs are not stored beyond the derived `task` string.

### 6.6 Focusing a session

Walk the process tree from the session PID up to the first ancestor that owns a visible top-level window (Windows Terminal, conhost, VS Code, Cursor, JetBrains, the Claude desktop app), restore it if minimised, and bring it to the foreground. Windows Terminal tabs cannot be targeted individually: the WT window is focused, not the tab. Documented limitation. If no window is found, the widget flashes the row instead.

## 7. Integration install

Triggered from the tray or a first-run banner. Never automatic.

1. Copy `~/.claude/settings.json` to `~/.claude/settings.json.peepthis-backup-<timestamp>`.
2. Merge hook entries for each event in §6.2. Existing user hooks are kept; Peep This's hook entries are identified by their `http://127.0.0.1:<port>/hook/` URL, so reinstall is idempotent and uninstall removes only them.
3. If `statusLine` exists, record its original command in `%APPDATA%\PeepThis\statusline-original.json` and replace it with `peepthis-statusline.exe`. If none exists, install the wrapper with no original (it then prints nothing).
4. Write with a temp file and atomic rename. Validate the result parses as JSON before replacing.
5. Uninstall reverses 2 and 3 exactly, restoring the recorded statusline command.
6. `integration: "outdated"` when installed hook entries carry a different port or token than `endpoint.json`, or the statusline entry points at a different binary path or version than the running app.

## 8. Performance and robustness budgets

- Idle: under 1% CPU, under 80MB RAM with 10 sessions.
- Peep animation runs only for visible rows (peep's own IntersectionObserver) and pauses entirely while the widget is hidden.
- File watching uses OS notifications, with a 5s poll as a safety net.
- Any malformed input (session file, hook body, transcript line) is skipped and logged, never fatal.
- Logs to `%APPDATA%\PeepThis\logs`, rotated, no prompt or tool content.

## 9. Testing

- `cargo test`: session file parsing (fixtures from real files, anonymised), state transitions (one fixture sequence per §4 row and rule), install merge and uninstall round trip on fixture settings files, endpoint token check.
- Vitest: identity hashing stability, sort order, formatting.
- Fixtures under `src-tauri/tests/fixtures/` and `src/**/__fixtures__/`.
- Manual check per surface ticket: run `npm run tauri dev` against live sessions and confirm what the owner sees.

## 10. Open questions

None blocking. The **Verify** items above are resolved inside their tickets and recorded there.

## 11. Mission control (planned, grill 2026-11-14)

The widget becomes the place a fleet of sessions is run from; the terminal becomes the detail view. Nothing here is built. Each subsection names its tickets, and the ticket that builds a part moves it into §3 to §9 and removes it from here. Build order: approvals (AV-20 to AV-26), subagents and shells (AV-27 to AV-30), rules (AV-31), digest (AV-32, AV-33).

### 11.1 Inline approvals (AV-20, AV-21)

- The widget answers `PermissionRequest` hooks instead of replying 204. Actions: allow, deny, deny with a typed reason that goes back to Claude as feedback.
- Widget first, then release: the request is held for a window (default 60s, configurable). With no answer the widget replies "no decision" and the normal terminal prompt takes over. It releases at once if that session's terminal is the foreground window, if the session moves on or dies, or if the widget hides or quits.
- Decisions originate only from the widget's own UI through a Tauri command. No HTTP route accepts a decision, so a leaked token can at worst show a fake card.
- Contract (AV-20, Claude Code 2.1.325): reply 200 JSON `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}`, or `"behavior":"deny","message":"<reason>"`. A 204, a timeout or a closed connection each mean no decision. Never send `updatedInput`, `updatedPermissions` or `interrupt` (an interrupt fires no `Stop`, so the row would hang).
- The `PermissionRequest` hook entry's `timeout` is the hold window + 5s (65 by default); other events stay at 1. `onFailure` stays unset (`block` turns a timeout into a deny). AV-08's `outdated` check compares this timeout too.
- After a widget deny no `PostToolUse` follows, so the engine leaves `Waiting` on the widget's own decision and goes to `thinking`. A connection that closes before the widget replies is a release: drop the card, send nothing.
- `PermissionRequest` has no `tool_use_id`. Join it to the latest unresolved `PreToolUse` with the same session, `agent_id`, tool name and identical input, and take that `tool_use_id` for the trail. Subagent requests use the same contract and carry `agent_id`.
- Owed to the AV-20 owner check: the terminal prompt while a hold is open and after its release, and a terminal answer racing the hook.
- Still out: answering questions, free-text replies, writing "always allow" rules into Claude Code's settings.

### 11.2 Risk tiers (AV-22)

A pure classifier puts every request in one of three tiers and fails closed (unknown is never `safe`).

| Tier | Examples | Guard |
|---|---|---|
| `safe` | Reads; edits inside the session cwd | One click; approvable from a toast |
| `shell` | Shell commands, edits outside cwd, MCP and unknown tools | One click with the full command shown |
| `destructive` | Recursive or forced delete, force-push, hard reset, publish/deploy, pipe-to-shell, writes to settings or credential files | Hold 1s to confirm; never from a toast; never auto-allowed |

Built in `risk.rs` (AV-22): `classify(tool, input, cwd) -> Verdict { tier, field, span, reason }`, where `span` is a byte range into `input[field]` for the card to highlight. Compound and nested commands take the highest tier of any part. Hidden commands (`-EncodedCommand`, nesting deeper than 6) fail closed upward to `destructive`. The full pattern list is in the ticket.

### 11.3 Approval card (AV-23, AV-25)

A session with a pending request opens a card under its row: a compact coloured diff of the real change for `Edit`/`Write`, the full command with the risky token highlighted and the cwd for shells, key arguments for other tools. Allow, Deny, Deny with reason, and a countdown to release. The waiting toast gains Allow and Deny for the `safe` tier only.

Built: the card in `src/approvalCard.ts` with pure helpers in `src/approval.ts` (AV-23); destructive Allow is a 1s hold, the last 3 deny reasons are kept locally. Toast buttons in `notify.rs` (AV-25): `safe` gets Allow/Deny, other tiers a line naming the inbox hotkey; the toast is built 200ms after the alert so the held request is visible, and a resolved toast is inert (no Action Center removal: the toast crate has no tag API).

### 11.4 Keyboard inbox (AV-24)

A global hotkey brings the oldest pending request across all sessions into focus. `Y` allow, `N` deny, `R` deny with reason, `Tab` next, `Enter` focus the terminal, `Esc` dismiss. Focus returns to the window the user came from.

### 11.5 Decision trail (AV-26)

Every allow, deny, release and auto-allow is logged locally with time, session, tool, target, tier and the rule that fired. The expanded row shows the last few; settings has the full searchable log. The trail stores commands and paths, never file contents: the one exception to §6.5 and §8, and it can be turned off.

### 11.6 Subagent child rows (AV-27, AV-28)

Replaces the count badge (§3.3) and the subagent rule in §4. Each running subagent is an indented child row with its own 32px peep in the parent's colour, an accessory hashed from the agent type, its own state and current tool. On finish it shows its result's first line, duration and tool count for 30s, then folds into a "done N" counter on the parent. More than 4 running collapse to "+N more". Peep art is used unchanged. The tracking behind it is built (AV-27, §4); the rows are AV-28.

### 11.7 Background shells (AV-29, AV-30)

Each background shell is a chip row under its parent with the command and runtime, in one of five states: `running`, `quiet` (no output for a while), `serving` (a listening port was found; the port is shown), `exited`, `failed` (exit code and last error line). A failed shell puts a red dot on the parent row that survives collapsing. Shells that outlive their session are out of scope. Tracking is built (AV-29, §4); `quiet` relies on the undocumented task output file path and degrades alone if that path moves. The chips are AV-30.

### 11.8 Suggested rules (AV-31)

After the third manual allow of the same shape (project + tool + normalised target) the widget offers to auto-allow it. Confirmed rules live in the widget's own store, are listed and revocable, and each use is written to the trail and shown as a tick. The `destructive` tier is never offered and never matched.

### 11.9 While-you-were-away digest (AV-32, AV-33)

On return from a locked screen or idle period, one card: blocked on you, failed, finished, still running, with time taken, files changed and commands run per session. Built from events only by default. An opt-in setting adds a one-sentence summary per session from a headless `claude -p` call on Haiku, which spends rate limit and needs the network.

### 11.10 Considered and not taken

Collision radar (two sessions on one file or branch), usage forecasts, phone remote, orphan shell tracking, a per-session trust dial, a global pause switch, launching sessions.

## 12. Surface (planned, surface grill 2026-11-14)

How the widget looks, collapses and docks. Nothing here is built; §3.1 describes the window as it is today. Each ticket that builds a part moves it into §3 and removes it from here. Build order: verify (AV-34), chrome (AV-35), bars and motion (AV-36, AV-37), snap (AV-38), strip (AV-39), then hotkey, peek, flyout and empty state (AV-40 to AV-43).

### 12.0 Pixel desktop shell (AV-44, owner design 2026-11-14)

The owner's "Pixel desktop" design (`design/shell/`) replaces the row and header look: a 90s desktop-pet window, striped title bar, square speech bubbles, each peep on a striped shelf plank. Rows: 48px peep, a bubble saying what the session is doing, name and an outlined state chip. Attention states (`waiting`, `error`, `stuck`) fill the bubble with the state colour and sort first. Clicking a row opens it in place on a 96px stage. 5h/7d sit small in the header as AV-36 block bars. Peep sizes 48 (rows), 96 (detail), 40 (strip) are the owner's ruling over the multiples-of-32 rule. Light theme is AV-45; the strip look folds into AV-39.

### 12.1 Pixel chrome (AV-34, AV-35)

Built: see §3.1. AV-34 recorded the platform answers.

### 12.2 Bars (AV-36)

The context bar and the 5h/7d limit bars are rows of 10 square blocks that fill one at a time: neutral, amber from 70%, red from 90%.

### 12.3 Motion (AV-37)

Every transition runs in 4 to 6 discrete frames over about 160ms, matching the peeps' 8 to 12 fps, including window resizes. With `prefers-reduced-motion` everything is instant.

Built (AV-37): 5 frames over 160ms. The tokens are `--motion-ms` and `--motion-steps` in `styles.css`, mirrored by `motion.ts`. Exceptions: the approval countdown and the destructive hold fill stay linear, because they show time passing.

**Exception, strip/panel switch (owner 2026-11-15, AV-39):** "fluid yet nostalgic" overrules the 4 to 6 frame / 160ms rule for this one transition. The window is never stepped. On a switch it is sized once to the union of the strip and panel rects with the anchored edge fixed, the card animates inside it (`fold.ts`: a frame travels between the two rects, strip and panel content cross-fade, clipped to the frame), and one `SetWindowPos` then drops the window to the final rect. `window.rs` (`begin_switch`, `play_region`, `end_switch`) keeps the hit region on the visible card throughout. Timing, chosen by the owner: 12 steps over 240ms. WebView2 shows a blank frame when its window resizes, so `cover.rs` holds a frozen picture of the strip's screen rect over the two resizes of a switch, until it sees the page's 2px beacon on screen. Reduced motion is instant.

### 12.4 Docking and snap (AV-34, AV-38)

- Live magnet: dragged within 16px of a work-area edge or corner, on any monitor, the widget jumps there with the 12px margin while still held; dragging more than 16px further breaks it free. Dropped elsewhere it floats.
- The snapped edge or corner is the anchor. It is saved with the position, and every size change keeps the anchored edge fixed.
- **Verify** (AV-34): a drag method that allows a live magnet. Fallback: snap on release.
- Built (AV-38): `snap.rs` holds the maths. `window.rs` subclasses the window and rewrites `WM_MOVING` from the cursor and the grab offset, because Windows chains each proposal from the last one. A drop part-way off the work area is pulled back inside. `window.json` stores `anchor` (`float`, an edge, or a corner like `bottom-right`). A file without one reads as `float`, or as `top-right` when it sits on the default dock.

### 12.5 Peep strip (AV-39)

- Three modes: `panel`, `strip`, `hidden`. The app launches in `strip`.
- The strip is a slim plate (62px, AV-44 design: square, 2px border, no border on the docked edge, each peep on a mini shelf, attention cells tinted and ringed in the state colour) with one 40px peep per top-level session, in the panel's sort order, capped at 8 and then a `+N` tile. No subagents or shells; a failed shell's red dot shows on its parent's peep.
- Orientation follows the anchor: a column on the left and right edges, a row on the top and bottom edges, a column at corners and when floating.
- Strip and panel share the anchor; the panel unfolds from the strip away from the snapped edge. The whole plate drags the widget.

### 12.6 Dismiss and recall (AV-40, AV-41)

- `Ctrl+Alt+P` (rebindable) toggles `panel` and `strip` without taking focus. A header collapse control does the same. No `Esc`, no click-outside.
- Tray left click toggles `hidden` against the last visible mode.
- Hovering the strip for 200ms unfolds the panel as a peek that folds back when the mouse leaves. A click pins it as `panel`.
- The keyboard inbox (§11.4) has its own key.

### 12.7 Waiting flyout (AV-42)

When a session enters `waiting` in `strip` mode, only that session's row slides out beside its peep, amber-lit. It leaves when the state leaves `waiting` or when clicked through to the terminal. The approval card (§11.3) mounts here when the widget is collapsed. Toast rules (§3.6) are unchanged.

### 12.8 Empty state (AV-43)

With no sessions, one house peep: `idle`, yellow, no accessory. 32px in the strip, 64px in the panel above "No sessions running". Art unchanged.

### 12.9 Considered and not taken

Single pill, edge tab, Mica/Acrylic backdrop, perch card, floating chips, state-tinted or accent shadow, ghost snap outline, auto-collapse on idle, auto-expand on waiting, `Esc` to dismiss.
