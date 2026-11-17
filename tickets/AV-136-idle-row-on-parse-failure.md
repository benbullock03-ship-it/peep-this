---
id: AV-136
title: Idle row with a ? chip on parse failure
status: done
depends: []
---

# AV-136: Idle row with a ? chip on parse failure

**What to build:** when a live session's file, hook payload or transcript fails to parse, the session stays listed from its PID with state `idle` and a `?` chip. A live session never drops off the list (AV-131 answer 9).

- [x] The backend keeps a session whose PID is alive even if its JSON fails to parse, and marks it `unparsed`
- [x] Its row shows `idle` with a `?` chip, and the tooltip says the format was not understood
- [x] A later good parse clears the mark
- [x] A test using the malformed fixtures: the session is still in the snapshot and marked

**Done when:** a corrupted session file in `npm run tauri dev` shows an idle row with `?`, and the row recovers when the file is fixed.

## Checks

1. Known session, corrupt file on two scans: kept, marked, old name/cwd: pass, `sessions.rs` `known_session_with_corrupt_file_stays_marked_unparsed`.
2. Corrupt file seen first with a live PID is listed from the PID and marked; dead PID not listed: pass, `sessions.rs` `unparsed_file_with_live_pid_is_listed` (plus `salvage_reads_complete_fields_and_falls_back_to_the_pid`).
3. A later good parse clears the mark: pass, check 1's test and `store.rs` `unparsed_mark_reaches_snapshot_and_clears`.
4. Snapshot holds the session `idle` with `unparsed: true`: pass, `store.rs` `unparsed_mark_reaches_snapshot_and_clears`.
5. Row view carries the `?` note ("format not understood") only when unparsed: pass, `rowView.test.ts` "carries the ? chip note only for an unparsed session"; the drawn chip and tooltip are owed to the owner.
6. Build and tests: pass, `npm run build && cargo check`; `npm test` 379 passed, `cargo test` 433 passed.
7. Live corrupt-then-fix in `npm run tauri dev`: pass (Big Peep, 2026-11-17): truncated fake `<pid>.json` on a live PID listed `idle`, `unparsed: true` within ~5s; row drew `?` beside IDLE with the tooltip and "format not understood" in the aria-label; valid JSON cleared it; fake file deleted.

## Done 2026-11-17

`sessions.rs` `scan_with` unparsed path + `salvage`, `Session.unparsed`, `rows.ts` `?` chip; tests in `sessions.rs`, `store.rs` `unparsed_mark_reaches_snapshot_and_clears`, `rowView.test.ts`; live check above.

## Decisions taken without a ruling

- A file is marked only after failing two scans in a row, so a file caught mid-write never flashes `?`.
- A session first seen with a corrupt file takes its id and cwd from complete `"sessionId"`/`"cwd"` strings in the broken text, else `unparsed-<pid>`; a later good parse with another id ends the made-up one.
- An unparsed row is listed from a live PID without the `procStart` check (it cannot be read), so PID reuse is not guarded while the file is broken.
- A known session that turns unparsed keeps its hook-driven state; only a session first seen unparsed starts `idle`. Forcing `idle` would hide a `waiting` prompt.
- An unparsed session that ends never reaches the closed shelf, since its id may be made up.
- Hook payloads and transcript lines were left alone: a malformed hook body carries no readable session id, and malformed transcript lines are already skipped; neither removes a session from the list.
- No Shoot run: no demo state shows an unparsed row, and the live check covers the chip.
