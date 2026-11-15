---
id: AV-78
title: Stale doing ticket after 3 days - worried peep, Ducks lists it first
status: done
depends: [AV-69]
---

# AV-78 — Stale doing ticket

**What to build:** Work that looks abandoned becomes visible without ever blocking. Parking with a reason counts as committing.

- [x] Backend: a ticket in `doing` whose file has not changed for 3 days (setup key `stale.days`, default 3) sets the worried mood on sessions in that repo, via the AV-69 mood engine and its priority order
- [x] Ducks lists stale `doing` tickets first, each with "finish, park with a reason, or close"
- [x] Nothing refuses to start new work
- [x] `cargo test` for the threshold and the setup override

## Decisions taken without a ruling

- Ducks now uses the ticket file's mtime and `stale.days` too (was: no `<id>:` commit in 7 days), so Ducks and the worried peep agree.
- Stale worry has no expiry; it lasts while the ticket stays stale and keeps worry's slot in the priority order.

## Done 2026-11-15

- `tickets.rs` `Repo.stale` (test `doing_tickets_unchanged_for_stale_days_are_stale`), `mood.rs` worry (test `a_stale_doing_ticket_worries_every_session_in_its_repo`), Ducks `gather.md`/`questions.md`, F§1 `stale.days`.
- Dev widget (`cargo run` against the running vite): with this file backdated 4 days, both sessions in this repo showed the worry emote and the Ponytail session in another repo did not; after `touch` back to now the emote cleared within one poll. Ducks' stale rule on the same files: AV-78 stale (4d), AV-46 fresh.

**Done when:** a `doing` ticket backdated 4 days makes that repo's peep worried in `npm run tauri dev`, and Ducks lists it first.
