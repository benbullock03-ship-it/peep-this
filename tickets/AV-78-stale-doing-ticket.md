---
id: AV-78
title: Stale doing ticket after 3 days - worried peep, Ducks lists it first
status: blocked
depends: [AV-69]
---

# AV-78 — Stale doing ticket

**What to build:** Work that looks abandoned becomes visible without ever blocking. Parking with a reason counts as committing.

- [ ] Backend: a ticket in `doing` whose file has not changed for 3 days (setup key `stale.days`, default 3) sets the worried mood on sessions in that repo, via the AV-69 mood engine and its priority order
- [ ] Ducks lists stale `doing` tickets first, each with "finish, park with a reason, or close"
- [ ] Nothing refuses to start new work
- [ ] `cargo test` for the threshold and the setup override

## Blocked

2026-11-15 batch: `src-tauri/src/mood.rs`, `store.rs` and `state.rs` hold another session's uncommitted clash-mood work. Wake when that lands. Plan: stale check in `mood::derive`'s worry condition, `stale.days` read like `tickets::setup_test`; Ducks line after.

**Done when:** a `doing` ticket backdated 4 days makes that repo's peep worried in `npm run tauri dev`, and Ducks lists it first.
