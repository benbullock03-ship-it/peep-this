# 0003 — The widget may end a session

Date: 2026-11-15 (Question Time). Status: accepted. Overturns "never kills sessions" from `CLAUDE.md` (Actions) and spec §2. Terms as in `CONTEXT.md`.

## Context

Until now the widget acted on a session only through permission decisions and Jump, and never killed one. Some live sessions have no window to Jump to, for example a Claude desktop scratch session: focus finds nothing, so the user has no way to close it short of Task Manager. The row stays on the widget until the process dies.

## Decision

- **The widget may end a session**, from the open row only, on a 1s hold (the same gesture as destructive approvals, spec §11.2). Never from the strip, a flyout, a toast or the keyboard inbox.
- Ending means `TerminateProcess` on the session's PID, through a Tauri command (`end_session`, AV-96) called only from the widget UI. It is never registered on the event server, so nothing reaches it over HTTP.
- It refuses when the PID's process start time no longer matches the session's `procStart`, so a reused PID is never killed.
- No new state: once the process is gone the existing lifecycle (AV-10) takes the row to done and fades it.
- Typing into a session, replying, answering questions and starting sessions stay out.

## Consequences

- The widget can now lose a user's work in flight. The hold, the open-row-only placement and the `procStart` check are what keep that deliberate.
- `TerminateProcess` is a hard kill: Claude Code gets no chance to flush or run `SessionEnd` hooks. The transcript up to the last write survives.
- Backend in AV-96, the hold control in AV-97.
