---
id: AV-90
title: Big Peep skill - run a ticket batch, ask stale little peeps
status: doing
depends: [AV-74]
---

# AV-90 — Big Peep skill

**What to build:** `/peep:bigpeep <ids>` runs a batch of tickets: the global `/batch` skill rebuilt as a Peep skill, reading the repo setup instead of guessing.

- [x] Dependency order from ticket `depends`; one little peep (subagent) per ticket; one commit per ticket, staging only its hunks
- [x] A blocked ticket gets parked with reason and wake-up condition, and the run moves on
- [x] Monitoring: a little peep with no file change, commit or message for 10 minutes is asked for a one-line status; Big Peep never stops it, and reports the answer to the owner
- [x] Ends with a report table (ticket, status, commit, notes) and a Pack Up note
- [x] No new widget UI: little peeps show as subagent child rows (AV-28)
- [x] Why line per AV-74
- [x] Add the skill to Walk's Extras block with a use-when line

**Done when:** a three-ticket batch with one dependency runs in order, makes three commits, and the report shows a status query for a deliberately slow ticket.

## Decisions taken without a ruling

- Skill lives in `plugin/skills/bigpeep/` (SKILL.md, `references/brief.md`, `watch.md`, `output.md`); no script, so no test.
- No new format in `formats.md`: the output table is in `references/output.md`; the run count key is `bigpeep`.
- An in-batch dependency that reaches `done` earlier in the run counts as met for the run (`depends` otherwise needs `closed`, which only the owner sets).
- Big Peep sets `doing` before spawning and `done`/`doing` after review; little peeps never set status, park, stage, commit or run the index. A little peep that hits a block reports `blocked: <reason>; wake: <condition>` and Big Peep parks it (park commit), plus any later batch ticket that depends on it.
- Little peeps run work.md steps 1, 3, 4 and 5.1 (they tick the checklist); Big Peep writes `## Done` or `## Owed before done` from the "Done when".
- Watch timer is a background `sleep 300`, so a status ask lands between 10 and 15 quiet minutes; asked again only after another 10. No reply is reported as "no answer", never a stop.
- Checks: `build` over the tree per wave, `test` once after the last commit (follow-up fix commit per ticket), as the global batch skill does.
- Report is printed, not saved to a file; `--unattended` and the batch-report script are dropped; `attended: true` tickets go with decisions recorded.
- Pack Up note via packup steps 1, 3 and 4 with no questions; it does not bump `packup` in `.peep/runs`.
- Why habit: finish before starting.

## Owed before done

- Live check per "Done when": a three-ticket batch with one dependency, three commits, a status ask for a deliberately slow ticket shown in the report. Owner runs it.
