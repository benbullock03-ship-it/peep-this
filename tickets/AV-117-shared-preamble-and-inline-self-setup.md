---
id: AV-117
title: Shared preamble and inline self-setup
status: open
depends: []
---

# AV-117 — Shared preamble and inline self-setup

**What to build:** One shared preamble every Peep skill cites, and no more "Run /peep:walk first" gate (Question Time 2026-11-16).

- [ ] `plugin/references/preamble.md`: read `.peep/setup.md`, `skills.off`, run count bump, Why line while the count is below 3. Each `SKILL.md` cites it in one line and drops its own copies
- [ ] Setup missing: the skill guesses it from the repo (as `walk/references/setup.md` step 1), shows a read-back of at most 5 lines, writes `.peep/setup.md` on a yes, then carries on with what was asked. A "no" stops with nothing written
- [ ] Walk is no longer required first; it stays as tour and customising (AV-118)
- [ ] Why lines and run counts are kept
- [ ] Update `plugin/README.md` and formats §1

**Done when:** in a repo with no `.peep/`, `/peep:ticket 1` offers the 5-line setup read-back, writes it on yes and goes on to the ticket; no skill prints "Run /peep:walk first"; `claude plugin validate plugin` passes.
