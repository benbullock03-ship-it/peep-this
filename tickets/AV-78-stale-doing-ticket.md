---
id: AV-78
title: Stale doing ticket after 3 days - worried peep, Ducks lists it first
status: doing
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

## Owed before done

- Owner check in `npm run tauri dev`: backdate a `doing` ticket 4 days (`touch -d "4 days ago" tickets/AV-NN-x.md`) and see that repo's peep worried; `/peep:ducks` lists it first.
- Built: `tickets.rs` `Repo.stale` (test `doing_tickets_unchanged_for_stale_days_are_stale`), `mood.rs` worry (test `a_stale_doing_ticket_worries_every_session_in_its_repo`), Ducks `gather.md`/`questions.md`, F§1 `stale.days`.

**Done when:** a `doing` ticket backdated 4 days makes that repo's peep worried in `npm run tauri dev`, and Ducks lists it first.
