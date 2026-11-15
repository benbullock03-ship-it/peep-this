---
id: AV-33
title: Opt-in model summary for the digest
status: in-progress
depends: [AV-32]
attended: true
---

# AV-33 — Opt-in model summary for the digest

**What to build:** Stub. A setting, off by default, that adds one model-written sentence per session to the away digest (spec §11.9).

- [x] Headless `claude -p` on the Haiku model, fed the session's event facts and last assistant message; never the transcript wholesale
- [ ] The summary session must not appear as a row or trigger hooks into the widget (built and unit-tested; live look owed)
- [x] Facts card renders first; sentences fill in when ready; failure or offline leaves the facts card as is
- [x] Setting text states that it spends rate limit and sends session facts to the model

**Done when:** the owner turned it on, returned to a digest with sensible one-line summaries, and saw no phantom session row.

Decide: attended because it spends the owner's quota and changes the offline promise.

## Owner rulings 2026-11-15

- Order: AV-32's live look first; AV-33 is built only once AV-32 is done.
- Data sent: per session the facts (group, time taken, file base names, command count, releases) and the 160-char first line the log already holds. The log stays paths-and-counts only; no full assistant message is kept.
- Calls: one `claude -p --model haiku` per digest, all sessions in one prompt, JSON back keyed by session id. Cap 12 sessions, 45s timeout.
- Isolation: the widget picks the `--session-id`, then drops that id and the child PID in hook ingest and the sessions-dir scan. `~/.claude/settings.json` is not touched. Verify live that no row appears.
- 2026-11-15: adopt the uncommitted work a lost session left, check it against this ticket, live-verify, commit.

## Built 2026-11-15

- `src-tauri/src/summary.rs`: one `claude -p --model haiku --tools "" --no-session-persistence --setting-sources project --strict-mcp-config --session-id <uuid>` per card, run in `%APPDATA%\PeepThis\summary`, 45s timeout, 12 entries. Entries go as `s1`..`sN`; session ids stay local. Tests: `prompt_keys_entries_and_sends_base_names_and_facts_only`, `parse_keeps_known_keys_and_one_clipped_line`
- `Store::ignore` drops the summary session in hook ingest, `ingest` and placeholders. Test: `ignored_session_never_becomes_a_row_or_a_held_request`
- `Away::fill` puts sentences on the card only if it is still the card they were written for. Test: `fill_reaches_only_the_card_it_was_written_for`
- Setting `digestSummary` in `settings.json`, off by default; checkbox and note in `src/settings.ts`
- The exact command was run by hand with one sample entry: exit 0, one JSON sentence back

## Decisions taken without a ruling

- The session is dropped by its id only, not by child PID as the ruling says. The session file carries the id, so the id covers the sessions-dir scan; `--setting-sources project` also keeps the user's hooks from firing at all.

## Owed before done

- Owner: turn the setting on, be away past the threshold, come back. Sentences should fill in under each name and no extra row should appear.
