---
id: AV-80
title: Backend records ended sessions and their end-state flags
status: closed
depends: []
---

# AV-80 — Ended sessions record

**What to build:** When a session's PID dies, the backend keeps a record for the Closed shelf that survives a widget restart or reboot.

- [x] On session end, write an entry to a local file in the app data dir: session id, name, cwd, colour and accessory, last bubble text, final state, ticket, ended-at
- [x] Flags computed at end: uncommitted changes in the cwd's repo, ticket still `doing`, no Pack Up note written during the session (`.peep/handoff.md` mtime), ended in `waiting`, `error` or `stuck`
- [x] Prune entries older than 8h on load and on write
- [x] Snapshot carries `closed: [...]` for the frontend
- [x] `cargo test` for each flag, the prune, and a restart round trip

**Done when:** a session closed with uncommitted edits still appears in the snapshot with that flag after the widget restarts.

## Decisions taken without a ruling

- Colour and accessory are not stored: the frontend derives both from `cwd` with the same hash as a live row, so the record carries `cwd` only.
- Final state and bubble are the session's as they were just before `Ended` (which resets them to `idle`/`success`), captured in the store and handed over outside the lock.
- File is `%APPDATA%/PeepThis/closed.json`, one JSON array, newest first; a session ending again replaces its entry.
- "Uncommitted" is any `git status --porcelain` output in the cwd, untracked files included; not a repo means no flag.
- "No Pack Up note" means `.peep/handoff.md` at the repo root is missing or older than the session's start (`born_at`).
- A resumed session keeps its closed entry; AV-82 decides what the shelf shows.

## Done 2026-11-15

- `src-tauri/src/closed.rs`: `Shelf` (load, prune, record) and `entry` (flags); `store.rs` `set_on_end`/`set_closed` and `Snapshot.closed`; wired in `lib.rs` `wire_closed`; `src/model.ts` `ClosedSession`.
- Tests: `closed::tests` (uncommitted, no Pack Up, attention, ticket doing, prune on load and write, restart round trip keeping the uncommitted flag) and `store::tests::ending_hands_over_the_session_as_it_was_and_closed_reaches_the_snapshot`. `cargo test` 391 passed, `npm test` 212 passed.
