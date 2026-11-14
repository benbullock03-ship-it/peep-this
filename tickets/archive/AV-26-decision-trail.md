---
id: AV-26
title: Per-session decision trail
status: done
depends: [AV-21]
attended: false
---

# AV-26 — Per-session decision trail

**What to build:** A local record of every approval outcome (spec §11.5), shown on the expanded row and searchable in settings.

- [x] Append-only JSONL at `%APPDATA%\PeepThis\trail\<yyyy-mm>.jsonl`: time, session id and name, cwd, tool, target (command or path), tier, outcome (`allowed`, `denied`, `released`, `auto-allowed`), deny reason, rule id when a rule fired
- [x] Stores the command or path, never file contents or diffs. This is the one exception to "no tool content on disk" (spec §6.5, §8); a setting turns the trail off
- [x] Expanded row shows the session's last 5 as ticks and crosses with tool and target; hover shows time and reason
- [x] Settings view: full log, filter by session, outcome and text
- [x] Retention: 90 days, pruned on startup
- [x] Tests: write, read back, prune, malformed line skipped

**Done when:** the owner saw their own allow, deny and a timed-out release for a live session listed correctly on its expanded row and in the log.

## Done 2026-11-14

Live check by Claude at the owner's request, in `npm run tauri dev` with WebView2 remote debugging (CDP clicks and keys go through the UI) and `PermissionRequest` POSTs shaped like Claude Code's to the hook port, on placeholder sessions `lc-*`:

- An allow (Read), a deny with reason (Bash, typed reason + Enter) and a 61s timed-out release all returned the right hook response (`allow`, `deny` + message, empty 204). They showed on each expanded row as ✓, ✗ and – with hover "19:50:39 · allowed", "denied / Reason: …", "released to terminal (timed out)".
- Settings log listed all of them. Session filter 1 of 5, outcome `released` 3 of 5, text and reason search 1 of 5 each.
- The header button toggles settings, and the tray menu "Decision log and settings" opens it from closed. The trail switch was off across a dev restart, then turned back on.
- Integration was already installed (hooks with `timeout: 75`); requests held the full 60s.

Follow-up, not blocking: two sessions with the same name (an old and a current `agentview-widget-2b`) look identical in the session filter.

## Decisions taken without a ruling

- No settings view existed, so a minimal one was built (`src/settings.ts`): a list-icon button at the right of the header toggles it in place of the session list, and a tray entry "Decision log and settings" shows the widget and opens it. It holds the trail switch and the log.
- The on/off switch lives in `%APPDATA%\PeepThis\settings.json` as `"trail": true|false` (default on), written read-modify-write so other keys survive and later settings can share the file. Off stops writing; existing entries stay and are still listed and pruned.
- Month files use UTC calendar months (no date crate; local month boundaries would need a Windows call). Display times are local.
- Every release is logged as `released` with a `via` cause: `timed-out`, `terminal-focused`, `moved-on`, `session-ended`, `answered-in-terminal` (Claude Code dropped the connection, including a widget click that arrived too late), `widget-hidden`, `widget-quit`. The hover title spells it out.
- Extra fields beyond the ticket: `via`, `agentId`. `rule` and the `auto-allowed` outcome are in the schema but nothing produces them until AV-31.
- Target is the first of `command`, `file_path`, `notebook_path`, `path`, `url`, `pattern`, `query`, clipped to 500 characters so a heredoc command cannot put file text on disk; MCP and other tools with none of these log an empty target. Deny reasons are clipped to 500.
- Released decisions show as a dash, not a tick or cross. Auto-allowed shows a tick with "auto".
- The trail is not in the `sessions` snapshot: the frontend fetches `trail_recent` (last 5 per session) at start and on each `trail` event, and `trail_log` when the settings view opens. Filtering is client side; the log renders at most 300 matches with a count line.
- Prune runs on a background thread at startup; it deletes whole months past the 90-day cutoff and rewrites the boundary month without old (and malformed) lines.
