---
id: AV-134
title: npm run capture for versioned fixtures
status: done
depends: []
---

# AV-134: npm run capture for versioned fixtures

**What to build:** one command that records live Claude Code payloads, anonymised, as contract-test fixtures, so checking a new release is one command (AV-131 answer 8).

- [x] `npm run capture`: reads the live `~/.claude/sessions/*.json` files, takes hook and statusline POSTs for a short window, and anonymises paths, names and ids
- [x] Writes to `src-tauri/tests/fixtures/<version>/`, with the version taken from the session file's `version`
- [x] Move today's flat fixtures into a folder named for the version they came from (or `legacy/` when unknown), and update the tests' paths
- [x] `cargo test` runs the parsers against every version folder
- [x] A test for the anonymiser

**Done when:** running `npm run capture` against a live session produces a new version folder, and `cargo test` passes over all of them.

## Decisions taken without a ruling

- Capture is a proxy: it points `%APPDATA%\PeepThis\endpoint.json` at itself for 60s (same token), forwards every POST to the widget, and restores the file only if it still names the capture port. Keeps at most 3 payloads per route.
- Only Claude Code payloads are versioned (`hooks/`, `sessions/`, `transcripts/` under `legacy/`); `settings/` and `tickets/` stay flat. The hand-made fixtures mix 2.0.0, 2.1.323 and 2.1.325, so they went to `legacy/`.
- Hooks go to the version folder of their session's file, else the newest live version; each live version gets its own folder. Unparseable session files (stale, truncated) are skipped.
- Besides paths, names and ids, the anonymiser redacts free text (`prompt`, `content`, `new_string`, `old_string`, `last_assistant_message`, `stdout`, `stderr`, `tool_response`) to `<redacted>`, so a capture never commits the user's code or words; `command` and `message` are kept, path-anonymised. Host name and user name are replaced too.
- The contract test lives in `src/source/claude/mod.rs` (`contract::every_version_folder_parses`) because `lib.rs` has another session's uncommitted edits; `malformed*` fixtures are skipped.

## Checks

1. pass: `npm run capture` against live sessions wrote `tests/fixtures/2.1.326/` (sessions + 9 hooks: PreToolUse, PostToolUse, statusline) and `tests/fixtures/2.1.325/sessions/`.
2. pass: `grep -rniE "benbu|agentview|datamap|desktop-emo|<live session id>|OneDrive"` over the new folders found nothing.
3. pass: `endpoint.json` byte-identical before and after the run (diff against a scratchpad copy); POSTs forwarded to the widget's port.
4. pass: `git mv` of `hooks/`, `sessions/`, `transcripts/` to `legacy/`; five test helpers point at `tests/fixtures/legacy/...`.
5. pass: `cargo test contract` runs `every_version_folder_parses` over `legacy`, `2.1.325`, `2.1.326`.
6. pass: `npx vitest run scripts/anonymise.test.mjs`, 3 tests.
7. pass with one failure from another session: build ok, `npm test` ok, `cargo test` 428 passed, 1 failed: `ingest::server::tests::test_command_runs_wrap_as_test_run_and_worry_the_session_live` asserts a `Worry` mood, broken by the uncommitted `src/mood.rs`/`src/store.rs` (AV-133 moods work); it reads its fixture from `legacy/` fine.

## Done 2026-11-17

`scripts/capture.mjs`, `scripts/anonymise.mjs` (+ `anonymise.test.mjs`), `npm run capture` in `package.json`, `src-tauri/tests/fixtures/{legacy,2.1.325,2.1.326}/`, contract test in `src-tauri/src/source/claude/mod.rs`.
