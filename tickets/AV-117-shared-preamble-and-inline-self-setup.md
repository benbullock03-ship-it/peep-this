---
id: AV-117
title: Shared preamble and inline self-setup
status: done
depends: []
---

# AV-117 — Shared preamble and inline self-setup

**What to build:** One shared preamble every Peep skill cites, and no more "Run /peep:walk first" gate (Question Time 2026-11-16).

- [x] `plugin/references/preamble.md`: read `.peep/setup.md`, `skills.off`, run count bump, Why line while the count is below 3. Each `SKILL.md` cites it in one line and drops its own copies
- [x] Setup missing: the skill guesses it from the repo (as `walk/references/setup.md` step 1), shows a read-back of at most 5 lines, writes `.peep/setup.md` on a yes, then carries on with what was asked. A "no" stops with nothing written
- [x] Walk is no longer required first; it stays as tour and customising (AV-118)
- [x] Why lines and run counts are kept
- [x] Update `plugin/README.md` and formats §1

**Done when:** in a repo with no `.peep/`, `/peep:ticket 1` offers the 5-line setup read-back, writes it on yes and goes on to the ticket; no skill prints "Run /peep:walk first"; `claude plugin validate plugin` passes.

## Decisions taken without a ruling

- Pack Up and Tweet keep running without a setup (no self-setup); Shoot self-sets up like the rest.
- No reply to the self-setup read-back (headless) counts as "no": nothing written, run not counted.
- `docs/spec.md` §13 and `CONTEXT.md` still say Ducks writes the setup; left alone (`CONTEXT.md` has another session's uncommitted edits).

## Done 2026-11-16

- `plugin/references/preamble.md`; every `SKILL.md` cites it and keeps only its Why line; `grep -rn "walk first" plugin` finds nothing.
- Live check: throwaway repo with one ticket `DM-01`, no `.peep/`, child `claude -p "/peep:ticket 1" --plugin-dir plugin`. Turn 1 printed the 5-line read-back and wrote nothing; turn 2 "yes" wrote `.peep/setup.md` (`Written by /peep:ticket`), worked DM-01 to `done`, committed, bumped `ticket: 1` and printed the Why line.
- `claude plugin validate plugin`: passed (only the existing author/licence warnings). Full run green: 369 vitest, 424 + 17 cargo.
