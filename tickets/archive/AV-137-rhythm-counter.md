---
id: AV-137
title: Rhythm counter and copy button on the Learned page
status: closed
depends: []
---

# AV-137: Rhythm counter and copy button on the Learned page

**What to build:** the rhythm counter (`CONTEXT.md`) on the Learned page. It counts sessions with a Pack Up, tickets closed through `/peep:ticket` and approvals from the widget. It stays local and is never sent anywhere (AV-131 answers 13 and 15).

- [x] The backend counts each habit from events it already sees (Pack Up runs, ticket closes, widget approval decisions) and saves the counts locally
- [x] The Learned page shows the three counts and the date of the first use of each
- [x] A "copy my rhythm" button puts a short plain-text summary on the clipboard for a beta issue
- [x] A test for counting and the summary text

**Done when:** a Pack Up, a ticket close and a widget approval each move their count, and the copy button gives the summary.

## Checks

1. Pack Up moves its count: pass in tests (`store::tests::rhythm_counts_pack_ups_per_session_and_tickets_closed_through_peep_ticket`, once per session); live: owed to the owner.
2. A ticket close moves its count: pass in tests (same test; a new `last_closed` whose id a repo session is linked to counts once, an unlinked closing does not); live: owed to the owner.
3. A widget approval moves its count: pass in tests (`approve::tests::widget_decisions_count_in_the_rhythm`: delivered allow and deny count, a dropped connection does not); live: owed to the owner.
4. Counts and first-use dates persist locally: pass (`learned::tests::rhythm_round_trips_with_first_use_dates`; stored under `learned.rhythm` in the widget `settings.json`, no route on the event server).
5. Learned page shows the counts and dates, summary text right: pass for the text (`learned.test.ts` "rhythmSummary"); the page itself owed to the owner (`?demo&learned` or the live Learned page).
6. Copy button puts the summary on the clipboard: owed to the owner (real clipboard only in `npm run tauri dev`).
7. Build and tests: pass (`npm run build`, `cargo check`, `npm test` 513 passed, `cargo test` 436 + 17 passed).

8. (added) Live in `npm run tauri dev` (Big Peep, 2026-11-17): pass. Throwaway `claude -p "/peep:packup"` moved packups 0 to 1; `claude -p "/peep:ticket close LC-01"` in a scratch repo moved tickets to 1 when `last_closed` changed; held `PermissionRequest` answered by a real click on Allow moved approvals to 1 (hook got `allow`); Learned page showed all three with first dates; "Copy my rhythm" click put the title plus three lines on the clipboard (`Get-Clipboard`).

## Done 2026-11-17

`learned.rs` `Rhythm`, `store.rs` `refresh_learned`/`linked_closings`/`count_approval`, `approve.rs` `Broker::decide`, `learned.ts` section and copy button; tests named above; live check 8.

Follow-ups seen, not in scope: first-use dates wrap mid-date at 360px panel width; a ticket close counts only if its session is live across the shelf reads before and after the archive.

## Decisions taken without a ruling

- Counts live in the existing `learned` setting (`learned.rhythm`) and reach the page through the existing `learned` command; no new command or route.
- A ticket close counts only when a session in that repo has the closed ticket's id as its ticket link (set by a `/peep:ticket` prompt), the same link the cheer mood uses.
- A session counts once for Pack Up however many times it runs it ("sessions with a Pack Up"); the per-session memory is not persisted, so a restart during a running Pack Up can count it again.
- Approvals count both Allow and Deny delivered from the widget; rule auto-allows, releases and dropped connections do not.
- The summary is a title line plus the three count lines with first-use day (local `YYYY-MM-DD`), no version or machine details.
