---
id: AV-103
title: Pack trim - fold Unpack into Ducks, drop hello, invocation diet
status: closed
depends: []
---

# AV-103 — Pack trim: fold Unpack into Ducks, drop hello, invocation diet

**What to build:** Fewer skills and less always-on context. Ducks already reads the Pack Up note first; it takes over Unpack's drift check and first-command run. Every model-invocable skill's description loads into every session, so only the light, in-the-moment ones stay invocable. Rulings: `reports/2026-11-16-reflective-pass.md` §5.

- [x] Ducks step 1: when `.peep/handoff.md` exists, run Unpack's repo check (branch, hash, uncommitted files, ticket statuses) and put drift in the read-back; on yes, run the note's `Then` before other Do lines
- [x] Remove `plugin/skills/unpack/` and `plugin/skills/hello/`; Pack Up's `First command` becomes `/peep:ducks`; update `note.md`, the Walk tour, `CONTEXT.md` (Unpack entry points to Ducks), spec §13 and any floppy or backend mapping for `unpack` and `hello`
- [x] `disable-model-invocation: true` on every skill except `ticket`, `packup`, `tweet`, `shoot`
- [x] Every description cut to one sentence plus its `/peep:<name>` trigger
- [x] Widget tests and fixtures that name `unpack` or `hello` updated

**Done when:** `plugin/skills/` holds 13 skills, only the four named are model-invocable, a Pack Up note followed by `/peep:ducks` reports drift and runs `Then` on yes, and `npm test` and `cargo test` pass.

## Decisions taken without a ruling

- Drift check lives in `ducks/references/drift.md`; a Pack Up note always gives a read-back (its `Then` is a Do line needing the yes), and drift touching `Then` drops that line with the reason under Loose.
- Floppy art column 10 (unpack) stays in the sprite sheet unused; `design/skill-pack-2/` reference left as is.
- CONTEXT.md: only the Unpack entry edited; the surrounding uncommitted glossary block is not this ticket's, so CONTEXT.md is left uncommitted.

## Done 2026-11-16

- `plugin/skills/` holds 13 skills; only ticket, packup, tweet and shoot lack `disable-model-invocation`.
- Live run: in a throwaway worktree with a drifted Pack Up note, a child `claude -p /peep:ducks` read back two drift lines (old hash 1413f4c against 891425f, plus an unlisted stray.txt) with `run /peep:tweet` first under Do. On `yes` (resumed session) it ran `/peep:tweet`, which reported the board empty.
- `npm test` and `cargo test` pass.
- Still with the owner: the CONTEXT.md Unpack entry is edited in the tree but uncommitted, because it sits inside the owner's uncommitted glossary block.
