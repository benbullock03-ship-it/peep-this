---
name: bigpeep
description: "Big Peep: run a batch of tickets in depends order, one little peep (subagent) per ticket and one commit each. Use when the user runs /peep:bigpeep <ids>."
disable-model-invocation: true
argument-hint: "<ids, e.g. AV-73 AV-75 or 73-77>"
label: Big Peep
---

# /peep:bigpeep

Big Peep reads, orders, briefs, watches, reviews and commits. Little peeps build. "F§1", "F§2" and "F§7" cite `${CLAUDE_PLUGIN_ROOT}/references/formats.md`; "work.md" and "park.md" are `${CLAUDE_PLUGIN_ROOT}/skills/ticket/references/`. Files in `references/` beside this file are read when their step runs.

## 0. Repo setup

1. Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count).
2. Ids: bare numbers take the prefix; `73-77` or `AV-73 to AV-77` is every id in that range that has a live ticket file. No ids: print the argument hint and stop.

## 1. Plan

1. Read every named ticket and each of its `depends` (live, then the archive by that id only). Order from the `depends` edges, not the ids.
2. `git status --short` and `git log --oneline -1`. Uncommitted files that are not this run's belong to another session. Read `.peep/tweets.md` (F§7) if there is one: open tweets touching a ticket's files go in its brief.
3. Mark each ticket:
   - **go**: every dep is `closed`, or is in this batch and reaches `done` earlier in this run.
   - **parked**: a dep outside the batch is not `closed`, or the ticket must change another session's file. Park it per park.md (`blocked`, a reason, a wake) and move on.
   - **skip**: `closed`, `done`, or parked with its wake not met; say which in the report.
   - `attended: true` tickets go; their decisions are recorded, not asked.
4. Waves: tickets with no edge between them and no file in common. In doubt, serial. A wave of one is normal.
5. Print the plan in four lines or fewer and start. Do not wait for a reply.

## 2. Run a wave

1. Set each ticket `status: doing`. Do not commit yet.
2. Spawn one little peep per ticket with the brief in `references/brief.md`. Little peeps follow work.md but never stage, commit or change the status.
3. A file two tickets need (the repo's `CLAUDE.md`, a shared config) is edited by Big Peep between waves, never by a little peep.
4. Watch every running little peep per `references/watch.md`: 10 quiet minutes earns one status ask, never a stop.

## 3. Close a wave

1. Read each report, then the diff. The report is a claim; the diff is the evidence. Run the little peep's tests yourself.
2. Run the setup's `build` over the whole tree. A failure traced to a ticket goes back to its little peep; one that was there before the wave is named, not fixed.
3. A little peep that reported blocked: park its ticket per park.md with the reason and wake it gave. Tickets later in the batch that depend on it are parked too: `wake: ticket:<it>`.
4. Status from the ticket's "Done when", not the report: every clause met, `## Done <YYYY-MM-DD>` and `done`; else `## Owed before done` and leave `doing`.
5. Run the setup's `index` command, if any.
6. Commit one per ticket, `<id>: <summary>`, staging its files by name, plus the ticket. Check `git log --oneline -1` first; if the branch moved, look at what landed. If the index file holds another session's rows, leave it uncommitted and say so in the report.
7. Tell the owner in two lines: what landed, and what the diff showed that the report did not.

## 4. Full check

After the last commit, run the setup's `test` once. A new failure is traced to its ticket and fixed in a follow-up commit `<id>: fix <what>`. A known failure is named.

## 5. Report and Pack Up

Report table exactly per `references/output.md`. Then write the Pack Up note: `${CLAUDE_PLUGIN_ROOT}/skills/packup/SKILL.md` steps 1, 3 and 4, no questions, `Then:` the next ticket from the report. Do not bump `packup` in `.peep/runs`.

## Never

- Push, release or deploy. Close a ticket: the batch is not the owner's yes (close.md).
- Stop a little peep, or ask a question that stalls the run. An open decision goes in the ticket; one that cannot be undone is asked once, at a wave boundary, with a recommended answer.
- Let two little peeps edit one file, or touch another session's uncommitted files.
- Add widget UI: little peeps already show as subagent child rows on the session's row.

## Last step: run count

Per the preamble: `Why: watch the diffs, not the reports; a report is a claim, the diff is the evidence.`
