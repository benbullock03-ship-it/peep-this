---
id: AV-131
title: "Grill: what takes Peep This from 7 to 9"
status: closed
depends: []
attended: true
---

# AV-131: Grill, what takes Peep This from 7 to 9

**What to build:** Nothing yet. The owner answers the rounds below; each answer becomes tickets or a ruling in `CLAUDE.md` (rating discussion 2026-11-16).

Settled already: the fleet view and Big Peep stay. They teach agent management early, so the audience is one person: a newcomer who grows into running a fleet. The 7 lost points on spread, format fragility, Windows-only reach and features shipped before they were proven.

**How to answer:** reply `agree all`, or `agree all except 3: <answer>, 7: <answer>`. Unnamed questions take the recommendation.

**Done when:** every question has an answer recorded under `## Answers`, and each answer that needs work has a ticket id beside it.

## Round 1: focus (cut what does not teach the rhythm)

1. **Earned cosmetics and the supporter toggle (AV-115):** build now, defer, or drop? Recommend: **defer** until there are real users to earn things. Cosmetics reward retention you can't yet measure.
2. **Sponsor link (AV-116, `doing`):** finish, or park until v1 ships? Recommend: **park**. A sponsor link before a release reads as asking before giving.
3. **Moods (built: AV-69 to AV-71):** keep, hide behind a setting, or remove? Recommend: **keep clash only**, because it teaches "two agents on one file". Hide the other moods behind a setting, off by default.
4. **Coderot:** keep in Extras, or move out of the pack? Recommend: **keep in Extras**. Test honesty is part of the rhythm, and it costs nothing when unused.
5. **A rule against churn:** a new skill or widget feature needs a named user problem and one real use before it gets art. Recommend: **yes**, written into `CLAUDE.md` under Decided. Reel, Scrub and progressive unlocks were all added and then removed.
6. **v1 cut line:** what ships first? Recommend: rows, waiting alerts, inline approvals, bars, the signpost (AV-129), subagent and shell rows, the six Core floppies and Big Peep. Everything else follows a release.

## Round 2: resilience (Claude Code formats change without notice)

7. **Format drift detection:** how should the widget notice when `~/.claude/sessions/*.json` or hook payloads change shape? Recommend: **a version gate**. Record the Claude Code `version` each fixture came from. On an unseen major or minor version, show one quiet "untested Claude Code version" line in the tray, never an error.
8. **Contract tests:** keep fixtures by hand, or capture them? Recommend: **a `npm run capture` script** that records live session, hook and statusline payloads (anonymised) into `src-tauri/tests/fixtures/<version>/`, so a new release is one command to check.
9. **Graceful degradation:** when parsing fails, what does a row show? Recommend: **the session stays listed from its PID with state `idle` and a `?` chip**. Never drop a live session from the list.

## Round 3: reach

10. **macOS:** when? Recommend: **straight after v1**. Tauri builds it. The work is window focus (AppleScript or the Accessibility API), the tray, and the always-on-top level, so plan one ticket per piece.
11. **Distribution:** where does a newcomer find it? Recommend: **the Claude Code plugin marketplace** for the pack, with the plugin's first run offering the widget download. One install path, matching ADR 0001.
12. **The pitch line:** keep "it tells you what to do next"? Recommend: **yes**, and make the README's first screen a 10-second GIF of the signpost Jump and an inline approval.

## Round 4: proof (does it actually teach?)

13. **Measure learning without telemetry:** how? Recommend: **a local-only "rhythm" counter** on the Learned page (sessions with a Pack Up, tickets closed through `/peep:ticket`, approvals from the widget), never sent anywhere. This keeps the "no accounts or cloud" non-goal.
14. **First outside users:** how many, and how? Recommend: **5 people new to Claude Code**, each watched once for 30 minutes on a first session. Fix the top 3 confusions before anything in Round 1 is reopened.
15. **The bar for 9/10:** what has to be true? Recommend: a newcomer installs it in one step, gets a Jump from the signpost in the first 5 minutes without reading docs, runs `/peep:ticket` and `/peep:packup` on their own by day 2, and nothing breaks across one Claude Code release.

## Answers

Owner, 2026-11-17: recommendations taken except 14 (unwatched open beta instead of watched sessions).

1. Defer AV-115 until there are real users. AV-132 (parked, wake: top 3 beta confusion issues fixed)
2. Park AV-116 until v1 ships. AV-132 (parked, same wake)
3. Keep clash only; other moods behind a setting, off by default. AV-133
4. Coderot stays in Extras. AV-132 (`CLAUDE.md`)
5. Churn rule, written under Decided. AV-132 (`CLAUDE.md`)
6. v1 cut line as recommended. AV-132 (`CLAUDE.md`)
7. Version gate with a quiet tray line. AV-135
8. `npm run capture` into versioned fixture folders. AV-134
9. Unparsed live session stays listed, `idle` with a `?` chip. AV-136
10. macOS straight after v1, one ticket per piece. AV-141, AV-142, AV-143
11. Plugin marketplace for the pack; first run offers the widget. AV-138
12. Keep the pitch line; README opens on a 10-second GIF. AV-139
13. Local-only rhythm counter on the Learned page. AV-137
14. Override: v1 ships as an unwatched open beta through the marketplace; testers prove the bar by pasting their rhythm counter. AV-140, AV-132 (`CLAUDE.md`)
15. 9/10 bar as recommended. AV-132 (`CLAUDE.md`), checked through AV-137 and AV-140
