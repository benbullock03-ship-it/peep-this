---
id: AV-09
title: Transcript tail fallback and API error signal
status: todo
depends: [AV-03]
attended: false
---

# AV-09 — Transcript tail fallback and API error signal

**What to build:** `source/claude/transcript.rs` per spec §6.4: tail each session's JSONL from a stored offset to fill model, last tool and API errors, and refine state for sessions without hooks.

- [ ] Locate the transcript from `cwd` + `sessionId` (**Verify** the slug rule against real paths and record it)
- [ ] First scan capped to the last 256KB; then offset-only reads on change
- [ ] Skip sessions with a hook event in the last 10s
- [ ] **Verify** the API-error marker in transcript lines and emit an `error` event from it
- [ ] Malformed lines skipped; fixtures from real transcripts, anonymised

**Done when:** `cargo test transcript` passes and a session started before hooks shows working/idle and its model in the widget.
