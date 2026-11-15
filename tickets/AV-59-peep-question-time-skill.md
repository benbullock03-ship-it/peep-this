---
id: AV-59
title: Question Time skill - batched grill, read-back, then tickets
status: done
depends: [AV-56]
attended: false
---

# AV-59 — `/peep:qt` ("Question Time with Peep")

**What to build:** The forward-looking grill for a new idea or problem.

- [ ] Rounds of 4 questions via AskUserQuestion, recommended option first, each standing on its own
- [ ] Looks facts up instead of asking; checks claims against the code and existing tickets and calls out contradictions
- [ ] Tugs loose ends: after each round, follows up on partial or "other" answers before moving on; each round offers "more questions" or "that's enough"
- [ ] Resolved terms go into `CONTEXT.md` as they land (glossary only)
- [ ] Read-back, then on yes drafts tickets in dependency order in the repo's ticket format; may stop after the read-back with nothing written

**Done when:** a run on a sample idea produces a read-back the owner confirms and tickets that match it.

## Decisions taken without a ruling

- Floppy label carried as an extra frontmatter key `label:`; `claude plugin validate plugin` accepts it.
- The "Keep going" question takes one of the 4 slots in every round while the frontier is open, so a round is 3 decisions + Keep going.
- Read-back groups fixed as Goal, Scope, Tickets, plus Open only for decisions the owner left undecided.
- "stop"/"no tickets" after the read-back ends with `peep: read-back kept, nothing written.`; nothing is saved.
- Ticket ids continue from the highest number across `tickets` and `archive`; the `index` command runs once after writing if setup has one.
- Shared formats are cited by path relative to the skill folder (`../../references/formats.md`), since the skill runs in the user's repo, not this one.

## Owed before done

- Headless proof (2026-11-15, scratch repo with the example `.peep/setup.md`): `claude --plugin-dir <plugin> -p "/peep:qt let customers save a card for faster checkout"` read setup, flagged that setup's area paths and WEB-12 are missing, and asked a round of 3 decisions + Keep going, recommended first (as text, since headless has no AskUserQuestion). Without setup it printed the `/peep:ducks` line and stopped.
- Owner: run it interactively and confirm the read-back and tickets. In any scratch repo with a `.peep/setup.md`:
  `claude --plugin-dir "F:/Projects/agentview widget/plugin"` then `/peep:qt let customers save a card for faster checkout`
