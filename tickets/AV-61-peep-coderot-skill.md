---
id: AV-61
title: Coderot skill - tiered rot in a throwaway worktree plus test-honesty checks
status: open
depends: [AV-56]
attended: false
---

# AV-61 — `/peep:coderot`

**What to build:** A check that the tests can actually fail.

- [ ] All rot happens in a temporary git worktree that's removed afterwards, even on failure; the owner's working tree is never touched
- [ ] Tier 1: stub out whole units in the target area (from the repo setup or argument); a unit whose removal no test notices is a survivor
- [ ] Tier 2, only where tier 1 was caught: small flips (condition, `<`/`<=`, off by one, early return, dropped await)
- [ ] Test-honesty checks: hollow tests (no assertion, or mock-called only), `.skip`/`.only`/`#[ignore]`/commented-out tests, code changed in recent tickets with no test touching it, flaky tests (suite run 3 times)
- [ ] Output: one terse ranked table; then offers to turn the top survivors into tickets in one batch, written only on yes
- [ ] Caps runtime: a fixed number of mutants per run, chosen by the owner's area argument

**Done when:** on this repo, a planted untested branch shows up as a survivor and the worktree is gone afterwards.
