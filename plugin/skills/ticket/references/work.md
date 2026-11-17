# /peep:ticket <id>: work one ticket

## 1. Read

1. Read the whole ticket.
   - `closed`: say so and stop.
   - `done`: it waits for the owner; say `/peep:ticket close <id>` closes it, and stop unless the owner asked for rework in this session.
   - `blocked` or `deferred`: if the wake-up condition is met (`ticket:` that ticket is `closed`; `date:` that date has passed) or the owner says it is, delete `reason` and `wake`, set `open`, and go on. Otherwise print its `reason` and `wake`, and stop.
2. Read each ticket in `depends` (live folder, then the archive by that id only). Any that is not `closed` blocks this one: park it per `park.md` with `status: blocked`, `reason: waiting on <dep>`, `wake: ticket:<dep>` (the first such dep), name the next ticket (below) in the report's `next` line, report, and stop. Never start the next ticket: one per run.

   **Next ticket.** Walk the depends tree depth-first in `depends` order, reading each id once (live, then archive by id; an archived ticket is `closed`). At each dep:
   - `closed`: satisfied, skip.
   - `done`: stop the walk; `next: /peep:ticket close <dep>`.
   - `open` with every one of its own `depends` `closed`: stop the walk; `next: /peep:ticket <dep>`.
   - `open` with a dep not `closed`, `blocked` or `deferred`: walk into its `depends`.
   - `doing`: in hand elsewhere; note it, skip.

   Walk ends with no hit: `next: none unblocked; nearest wake <dep> <wake>`, from the parked deps in the tree: the earliest `date:`, else the first `ticket:` in walk order, else the first free text. No parked dep, only `doing`: `next: none unblocked; <dep> is doing`.
3. `git status --short` and `git log --oneline -1`. Uncommitted files that are not yours belong to another session. If the ticket must change one, park it `blocked` (reason names the file, wake is free text) and stop.
4. Area note: from the paths the ticket names, or the files you expect to touch, pick the one area (SKILL.md §2) and read `kb/<area>.md`. No match or no note: go on without one.
5. Tweet board: read `.peep/tweets.md` (F§7) if there is one. For each tweet touching the files you expect to touch, read it before building: an open `clash` with your plan gets a reply or a tweet of your own (`/peep:tweet`); never wait on one.

## 2. Start

One edit to the ticket, before any other file changes:
1. Set `status: doing`.
2. Add `## Checks` at the end: the ticket's "Done when" (and any checklist item that can be tested) as a numbered list of pass/fail checks, each `<n>. open: <check>; evidence: <test name, command or shot path>`. Banned: "works", "looks right" with nothing named.

Then print the same list in a reply line that starts `Checks:`. Do not commit yet.

## 3. Build

- Do what the ticket's checklist says, nothing beyond it.
- Run the setup's `build` after each meaningful step. Fix what you broke; name what was already broken, do not fix it.
- UI changed and the setup has a `url`: run `/peep:shoot` on what changed before calling it done.

## 4. Check

Run the setup's `test` once the build is finished. Name each failure as one of:
- new (yours): fix it;
- known: fails the same way on a clean `HEAD`;
- another session's: caused by an uncommitted file that is not yours; name the file, do not fix it.

A proof that rests on what the owner sees (a prompt, a page in their browser) is not proven by a command returning: put it under owed.

## 5. Finish

1. Tick the checklist items that are actually done; leave the rest.
2. Answer each check under `## Checks` in place: replace `open` with `pass`, `fail` or `owed to the owner`, and the evidence with what answered it. A check added after the work started is marked `(added)`.
3. No check is `fail` and "Done when" fully met: add `## Done <YYYY-MM-DD>` naming what in the tree proves it (file, test, command), then set `status: done`.
   Any `fail`, or not fully met: add `## Owed before done` with exactly what is missing, and leave `status: doing`. `owed to the owner` checks go under `## Owed before done` too.
4. Run the setup's `index` command, if any.
5. Stage only this ticket's files by name, plus the ticket and the index file. Check `git log --oneline -1` again; if the branch moved, look at what landed first.
6. Commit `<id>: <short summary>`.
