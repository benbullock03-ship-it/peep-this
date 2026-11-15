---
id: AV-61
title: Coderot skill - tiered rot in a throwaway worktree plus test-honesty checks
status: closed
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

## Evidence (2026-11-15)

Scratch clone of this repo at HEAD plus a commit adding `.peep/setup.md` (`test: npx vitest run src/rot`, `area.rot: src/rot/`) and `src/rot/clamp.ts` with an untested `Number.isNaN` branch. Ran `claude --plugin-dir <repo>/plugin -p "/peep:coderot rot" --permission-mode acceptEdits --add-dir <TEMP> --allowedTools "Bash(git:*)" "Bash(npm:*)" "Bash(npx:*)" "Bash(mktemp:*)" "Bash(cd:*)" "Bash(rm -rf:*)" Read Edit Write Glob Grep`:

```
Coderot: rot, 4 mutants (cap 20), 1 survivors, HEAD a5d9cb6
| 1 | survivor t2 | src/rot/clamp.ts:4 | `if (Number.isNaN(n)) return 50;` -> `if (false) return 50;` |
worktree removed
```

Afterwards `git worktree list` in the clone showed only the main tree and `git status --short` was empty. The yes-path (ticket writing) was not exercised headless.

## Decisions taken without a ruling

- Arguments are `<area or path prefix> [cap]`; cap defaults to 20 mutants, max 50; no argument lists the areas and stops.
- Any flaky baseline run skips rot (a catch could be the flake) but still runs honesty checks; a red baseline skips both.
- Dependencies are installed fresh in the worktree (`npm ci` etc.); the owner's `node_modules`/`target` are never linked in, since a forced remove could follow the link.
- A stub that won't compile is not a catch and does not count against the cap.
- Ranking: survivor t1, survivor t2, only, flaky, skipped/commented-out, hollow, untested change; ticket offer covers at most 5 survivor/only/flaky rows.
- "Recent tickets" for the untested-change check means the last 10 commits touching the target.
- Output reference is `references/output.md` (a hook blocks subagents writing files named `report.md`).
