---
id: AV-76
title: Unpack skill - check the Pack Up note, read back, start
status: done
depends: [AV-74]
---

# AV-76 — Unpack skill

**What to build:** `/peep:unpack` opens a session from `.peep/handoff.md`, so carrying on is a real step and not "do the handoff".

- [ ] Reads the Pack Up note; no note: one line saying so and suggests `/peep:ducks`
- [ ] Checks the repo still matches: branch, commit, uncommitted files, ticket statuses named in the note
- [ ] Read-back (formats §4) with anything that drifted
- [ ] On "yes", runs the note's first command; on a correction, a fresh read-back
- [ ] Pack Up's note ends with `First command: /peep:unpack` when the next step is clear (edit `plugin/skills/packup`)
- [ ] Why line per AV-74

## Decisions taken without a ruling

- The note format lives in `skills/packup/references/note.md` (not formats.md), outside this ticket's allowed files, so Pack Up's rule is stated in `packup/SKILL.md` step 3 only. note.md still lists `First command: <the exact command>` followed by a `Then:` line, so `First command: /peep:unpack` is not literally the last line; wants a note.md tweak.
- Unpack runs the First command; when that is `/peep:unpack` itself it runs the `Then` line (else "Left to do"), to avoid a loop.
- Unpack stops without counting a run when no note exists.
- Drift touching the command (ticket already `done`, branch changed) makes Unpack wait rather than run on "yes".

**Done when:** after `/peep:packup`, a new session running `/peep:unpack` reports drift from a commit made in between and starts the first command on yes.

## Owed before close

- Live check of the Done-when above. Not run in the batch. Template line for `First command` updated in `plugin/skills/packup/references/note.md`.
