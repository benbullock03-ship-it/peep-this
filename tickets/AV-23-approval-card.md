---
id: AV-23
title: Approval card on the session row
status: in-progress
depends: [AV-21, AV-22]
attended: false
---

# AV-23 — Approval card on the session row

**What to build:** The row surface for inline approvals (spec §11.3). A session with a pending approval opens a card under its row showing exactly what is being approved, with Allow, Deny and Deny with reason.

- [x] `Edit`/`Write`: compact coloured diff of the real change, monospace, scrollable, capped height. `Bash`/`PowerShell`: full command, wrapped, risky span from AV-22 highlighted, cwd shown. Other tools: key arguments as a short list
- [x] Allow / Deny / Deny with reason (inline text field, Enter sends, last 3 reasons offered)
- [x] `destructive` tier: Allow is press-and-hold for 1s with a visible fill; a click does nothing
- [x] Countdown to release (thin bar) so the user knows when the terminal takes over
- [x] A waiting row with a pending approval auto-expands its card; this overrides "one row expanded at a time" only for cards
- [x] Several pending on one session stack oldest first
- [x] Card closes the moment the request resolves from anywhere (decided, released, answered in terminal)
- [x] Peep and name still focus the terminal
- [x] Vitest: diff building, argument formatting, hold-to-confirm timing; `?demo` gains one card of each tier

**Done when:** the owner allowed an edit, denied a command with a reason that Claude then acted on, and hold-confirmed a destructive command, all without touching the terminal.

Decide: diff rendering (recommended: a small in-repo line differ over `old_string`/`new_string`; no dependency).

## Built 2026-11-14

- `src/approval.ts` (pure): `lineDiff` (trim common head/tail, LCS in between, delete-then-add above 250k cells), `diffFor` for Edit, MultiEdit, Write and NotebookEdit, capped at 200 lines with a "N more" line; `formatArgs` (6 keys, 160 chars); `byteSpanToChars` turns risk.rs UTF-8 byte spans into JS indexes; `riskParts`, `displayPath`, `byOldest`, `pushReason`, `createHold`.
- `src/approvalCard.ts`: one card per pending request inside the row, keyed by request id so a half-typed reason survives snapshots. Shell tools show the wrapped command with the span marked, the description and cwd; file tools show the diff (160px cap, scrolls both ways); others a key/value list. Tier chip carries the classifier reason as a tooltip. Countdown is a 2px bar drained by a CSS animation with a negative delay, so it needs no timer. Destructive Allow is "Hold to allow": pointer or Space/Enter held 1s with a fill; a click, early release or blur does nothing. Deny with reason opens an inline field (Enter sends) with the last 3 reasons as chips.
- `src/rows.ts`: rows render cards whenever `pending` is non-empty on a live session, independent of the one expanded row. Clicks inside cards never toggle or focus; peep and name still focus. Cards drop the moment a snapshot no longer lists the request.
- `src/demo.ts`: `?demo` holds one card per tier (ingest-01 shell, triage-03 safe Edit, scrape-06 destructive), re-armed instead of released; decisions clear them through `demoDecide`.
- `src/devApprovals.ts` removed.
- Tests: `src/approval.test.ts` (diff, args, spans, paths, ordering, reasons, hold timing with fake timers), demo test in `src/rowView.test.ts`.

## Decisions taken without a ruling

- Diff: in-repo line differ, no dependency, as recommended. Lines only, no intra-line highlight. Write shows the whole new content as additions, since the old file is not in the request.
- Clicking a recent-reason chip fills the field rather than sending, so a stray click cannot deny.
- Last 3 reasons live in `localStorage` (`peepthis.denyReasons`); they are a convenience, lost if WebView2 storage is cleared.
- Styling follows the current rounded chrome; AV-34/35 will restyle it with the rest of the panel.
- A failed `decide_approval` call re-enables the card; a `false` (already released) leaves it disabled until the next snapshot removes it.

## Owed before done

- Owner, in `npm run tauri dev` after reinstalling the integration from the tray (hook timeout 75): allow an edit, deny a command with a reason Claude then acts on, and hold-confirm a destructive command, all from the card. Also: does a stacked second request and the countdown read right.
- Checks so far: `npm run build` clean, 56 Vitest, 229 + 15 cargo tests. Shots of `?demo` at 360px looked at: no overlap, command wraps, diff scrolls sideways.
