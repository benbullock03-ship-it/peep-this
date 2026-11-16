---
id: AV-109
title: Signal tips with a Pack Up Jump at the context line
status: done
depends: []
---

# AV-109 — Signal tips with a Pack Up Jump at the context line

**What to build:** The widget teaches how models work at the moment it matters: a one-line bubble tip on a real signal, once each, never while `working` or `thinking`. Facts and sources: `reports/2026-11-16-reflective-pass.md` §4.

- [x] Verify first: does the statusline payload carry absolute context tokens (or window size) as well as %? Record in this ticket. % alone misleads on 1M windows
- [x] Context line (~120k tokens, else % of window as a fallback): "Getting crowded; answers get fuzzier from here." with a Pack Up Jump
- [x] Same tool failed twice: "Two misses? /clear with what you learned beats a third try."
- [x] Idle longer than the cache lifetime: "Cache went cold; the next reply re-reads everything."
- [x] First subagent: "That helper reads in its own window and sends back a summary."
- [x] Auto-compact seen: "Compacted: CLAUDE.md and recent files kept, details lost. That is why Pack Up exists."
- [x] Each tip once per session (context line) or once ever (the rest); dismissible; attention states take the bubble first
- [x] Rust tests for each trigger in the state engine; Shoot both themes

**Done when:** each tip fires once on its fixture signal, never during working/thinking, and the context-line Jump copies `/peep:packup` and focuses the terminal; the owner confirms one live tip.

## Verified: statusline payload (2026-11-16)

Yes. Per the Claude Code statusline docs, `context_window.total_input_tokens` (input + cache read + cache write from the last response) and `context_window.context_window_size` are present, and `prompt_cache.ttl` (`"5m"`/`"1h"`, v2.1.251+) gives the cache lifetime. The local `statusline.json` fixture now carries all three. Crowded keys on `total_input_tokens >= 120000`, falling back to 60% when no token count arrives.

## How it works

- `state::apply` raises `Session.signal`: crowded on a statusline, misses on a second `PostToolUseFailure` of one tool since the last prompt (`Event::ToolFailed`, main session only, interrupts excluded), cold on a `Tick` once idle or done past the TTL (default 1h) after a request (`Session::cold_due`, which also gates the ticker), helper on `SubagentStart`, compacted on a transcript `compact_boundary` with `trigger: auto` (live reads only).
- `store.rs` `refresh_tips` runs in every `mutate`: a signal becomes `Session.tip` unless one is already showing or a once-ever tip was seen; seen tips persist in the widget's `settings.json` under `tipsSeen` (`tip.rs`). A prompt or `dismiss_tip` clears the tip.
- The row (`src/tip.ts` `tipFor`) shows the tip only while `idle` or `success`, with no nudge; text in the dashed suggestion bubble (now up to 4 lines), Pack up (crowded only) and Got it under the plank. `?demo&tip=<name>` for Shoot.

## Decisions taken without a ruling

- A tip counts as seen when it fires, not when the user looks at it; a once-ever tip missed while away is gone. Simpler than tracking what reached the screen.
- A signal that arrives while another tip is showing is dropped and not marked seen, so it can fire again on a later signal.
- Manual `/compact` raises nothing; only auto-compact teaches.
- Suggestion bubbles (nudge and tip) clamp at 4 lines instead of 2: the helper and compacted texts were cut mid-sentence at 2.

## Live check (2026-11-16, run by Claude)

In `npm run tauri dev` with CDP:

- A real crowded tip was already raised on a busy session from its own statusline and stayed hidden while it was `thinking`/`working`.
- A throwaway CLI session reached `success`. A statusline POST with `total_input_tokens: 140000` raised `tip: crowded`. The row showed "Getting crowded; answers get fuzzier from here." in the dashed bubble, with Pack up and Got it under the plank.
- A CDP mouse click on Pack up put `/peep:packup` on the clipboard. The foreground window became that session's Windows Terminal tab. The tip stayed up after the click; only Got it or a prompt clears it.

Owed: the owner rules on the two decisions above, then closes the ticket.
