---
id: AV-109
title: Signal tips with a Pack Up Jump at the context line
status: open
depends: []
---

# AV-109 — Signal tips with a Pack Up Jump at the context line

**What to build:** The widget teaches how models work at the moment it matters: a one-line bubble tip on a real signal, once each, never while `working` or `thinking`. Facts and sources: `reports/2026-11-16-reflective-pass.md` §4.

- [ ] Verify first: does the statusline payload carry absolute context tokens (or window size) as well as %? Record in this ticket. % alone misleads on 1M windows
- [ ] Context line (~120k tokens, else % of window as a fallback): "Getting crowded; answers get fuzzier from here." with a Pack Up Jump
- [ ] Same tool failed twice: "Two misses? /clear with what you learned beats a third try."
- [ ] Idle longer than the cache lifetime: "Cache went cold; the next reply re-reads everything."
- [ ] First subagent: "That helper reads in its own window and sends back a summary."
- [ ] Auto-compact seen: "Compacted: CLAUDE.md and recent files kept, details lost. That is why Pack Up exists."
- [ ] Each tip once per session (context line) or once ever (the rest); dismissible; attention states take the bubble first
- [ ] Rust tests for each trigger in the state engine; Shoot both themes

**Done when:** each tip fires once on its fixture signal, never during working/thinking, and the context-line Jump copies `/peep:packup` and focuses the terminal; the owner confirms one live tip.
