---
id: AV-09
title: Transcript tail fallback and API error signal
status: done
depends: [AV-03]
attended: false
---

# AV-09 — Transcript tail fallback and API error signal

**What to build:** `source/claude/transcript.rs` per spec §6.4: tail each session's JSONL from a stored offset to fill model, last tool and API errors, and refine state for sessions without hooks.

- [x] Locate the transcript from `cwd` + `sessionId` (**Verify** the slug rule against real paths and record it)
- [x] First scan capped to the last 256KB; then offset-only reads on change
- [x] Skip sessions with a hook event in the last 10s
- [x] **Verify** the API-error marker in transcript lines and emit an `error` event from it
- [x] Malformed lines skipped; fixtures from real transcripts, anonymised

**Done when:** `cargo test transcript` passes and a session started before hooks shows working/idle and its model in the widget.

## Decisions taken without a ruling

- **Slug rule (verified 2026-11-13):** the project directory is the cwd with every char that is not ASCII alphanumeric replaced by `-`, case kept. Evidence: all 20 directories under `~/.claude/projects/` fit, e.g. `F:\Projects\agentview widget` → `F--Projects-agentview-widget` (`:` and `\` each become `-`, space too) and a `datamap.cymru` path → `datamap-cymru`. If `<slug>/<sessionId>.jsonl` is missing, every project directory is searched for `<sessionId>.jsonl`, which covers long cwds whose slug Claude Code truncates and hashes.
- **API-error marker (verified):** top-level `"isApiErrorMessage": true` on a `type: "assistant"` line whose `message.model` is `"<synthetic>"`, with a top-level `error` code (`rate_limit`, `server_error` seen) and sometimes `apiErrorStatus` (429). 16 such lines exist locally, in main and subagent transcripts. Summary = first non-empty line of the message text, else `API error: <code> (<status>)`, cut to 80 chars. `isApiErrorMessage: false` synthetic lines are ignored, as is the `<synthetic>` model.
- **Hook-fresh skipping lives in the state engine, not the tailer.** The tailer reads every tracked transcript so its offset stays current (skipping reads would replay a stale backlog once hooks go quiet); `state::apply` drops `TranscriptToolUse`/`TranscriptToolResult` within `HOOK_FRESH_MS` (10s) of a hook. API errors and the model always apply.
- **First scan replays nothing stale:** it emits only the latest model and, if the last tool use has no result, that tool (so a pre-hooks session mid-tool shows `working`). Old tool events and old API errors are not replayed; live reads emit every event in order.
- **Model:** new `Event::TranscriptModel { model }`, metadata only (no state change, no stuck-timer reset). The id is shown as a display name (`claude-opus-5-5` → `Opus 5.5`). Once a statusline has supplied a model, the statusline value wins so the two sources do not flip-flop.
- **Transcript tool uses never change the subagent count.** `SubagentStop` only comes from hooks, and a transcript line racing its `PreToolUse` would double-count.
- Sidechain (`isSidechain: true`) lines and `subagents/*.jsonl` files are ignored. Tool target: `file_path`/`notebook_path` (relative to cwd when inside it), first line of `command`, else `pattern`, `description`, `url`, `query`, `skill`, `path`.
- Rotation: a shorter file or a changed creation time resets the tail to a fresh capped first scan.

## Done 2026-11-13

Owner confirmed a session started before the integration was installed showed its state and model. `source/claude/transcript.rs`; `cargo test transcript` passes (21).

The check that was owed:

- Owner check in `npm run tauri dev`: a session started before hooks shows working/idle and its model. `cargo test transcript` passes (21).
