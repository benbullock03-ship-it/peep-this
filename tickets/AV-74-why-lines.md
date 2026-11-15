---
id: AV-74
title: Why line in every Peep skill, dropped after 3 runs
status: done
depends: []
---

# AV-74 — Why line in every Peep skill, dropped after 3 runs

**What to build:** Each Peep skill ends with one line on why its habit matters, so a new user learns the reason as well as the step. The habits taught are think before code, finish before starting, and prove it works.

- [ ] One why line per skill, written into each `SKILL.md` (ducks, qt, ticket, shoot, packup, coderot), tied to one of the three habits
- [ ] Run count per skill per machine in `.peep/runs` (flat `skill: n` lines); the why line shows while the count is below 3
- [ ] Document the `.peep/runs` file in `plugin/references/formats.md` §1
- [ ] New skills from AV-75 onward follow the same rule (note in `plugin/README.md`)

**Done when:** a skill run three times in a fresh repo shows its why line on runs 1 to 3 and not on run 4.

## Decisions taken without a ruling

- The shared rule lives in `formats.md` §1 ("Run counts"); each skill has a short last step citing it plus its own why text.
- Runs that stop early (no setup, bad argument) do not count.
- Why line is the one allowed addition to the fixed output formats (after the last line).
- Habits: ducks finish, qt think, ticket finish, shoot prove, packup finish, coderot prove.
- `.peep/runs` is not auto-ignored; skills mention a missing `.gitignore` entry rather than editing it.

## Owed before close

- Live check: one skill run four times in a fresh repo; Why line on runs 1 to 3, not on run 4. Not run in the batch.
