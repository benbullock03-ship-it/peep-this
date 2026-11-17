---
id: AV-121
title: Slim Ducks
status: closed
depends: [AV-120]
---

# AV-121 — Slim Ducks

**What to build:** Ducks is loose ends, read-back, act. Nothing else (Question Time 2026-11-16).

- [x] Remove depth levels 2 to 5 and the `Depth` question; `references/depth.md` goes. Nothing loose and no Pack Up note: straight to the output with `next: clear to proceed`
- [x] Hot spots and pattern cards are Goose's (AV-120); when nothing is loose, the output may add one line `deeper look: /peep:goose health`
- [x] Update spec §13, `CONTEXT.md` and anything that cites Ducks report cards (AV-105, AV-110 wording)

**Done when:** `/peep:ducks` in a clean repo asks nothing and prints the three-line output; in this repo it still finds loose ends, reads back and acts on yes; `SKILL.md` is shorter than today's 70 lines.

## Checks

1. Clean repo asks nothing: pass. `plugin/skills/ducks/SKILL.md` §3 has no AskUserQuestion; `grep -n -i depth` on it finds nothing; `references/depth.md` deleted. Live run owed to the owner.
2. Clean-repo output is three lines ending `next: clear to proceed`: pass. SKILL.md Output: `peep ducks: 0 loose ends, 0 acted on`, `deeper look: /peep:goose health`, `next: clear to proceed`. Live run owed to the owner.
3. In this repo it still finds loose ends, reads back and acts on yes: owed to the owner. Steps 1, 2, 4 to 6 and `gather.md`, `questions.md`, `act.md`, `drift.md` are unchanged.
4. `SKILL.md` shorter than 70 lines: pass. `wc -l` gives 65.
5. Hot spots and pattern cards are Goose's only: pass. `grep -rn -i "hot spot\|candidate card\|top pick" plugin/skills/ducks` hits only §3's pointer to Goose and the schema in `references/report.md`.
6. Spec §13, `CONTEXT.md` and AV-105/AV-110 wording repointed: pass. `docs/spec.md` §13, `CONTEXT.md` Ducks, `kb/plugin.md`, `src/learned.ts`, `src-tauri/src/learned.rs` now name Goose health; grep for `ducks.*(report card|level [2-5]|depth|candidate)` finds only the new AV-121 line in kb/plugin.md.
7. map.mjs `--report` dead code removed: pass. `renderReport`, `fixBrief`, `diagram`, the report CSS and the `--report` flag are gone; grep finds no caller in `plugin/`, `src/` or `src-tauri/src`; `npx vitest run plugin/skills/goose plugin/skills/ducks --exclude "src-tauri/**"` passes (10 tests).
8. Build and tests: pass. `npm run build`, `cargo check`, `npm test` (369 passed) and `cargo test` (422 + 17 passed).

## Live runs (2026-11-16, `claude -p --plugin-dir plugin` in scratch repos)

- Clean repo, first run: asked nothing and wrote only `.peep/runs`, but added a fourth line, `.gitignore lacks .peep/runs`. The same note came back in a second clean run. Fix: `references/gather.md` now says `.peep/runs` and `.peep/handoff.md` are the pack's own files, never a loose end and never mentioned. A rerun printed exactly the three lines plus the Why line.
- Repo with loose ends (a `done` ticket LO-1, an open LO-2 that depends on it, and a Pack Up note whose `Then:` is `/peep:ticket LO-2`): Ducks gave the read-back with `close 1 done: LO-1 (unconfirmed)` and `run /peep:ticket LO-2 (after the close)`, then stopped. On "yes" it marked LO-1 closed and staged the `git mv` to the archive. It left `/peep:ticket LO-2` unrun because the close had not committed, and ended with `next: /peep:ticket LO-2`. The commit failed only because the scratch repo had no git identity, and Ducks reported that.
- This was run in a scratch repo, not this one, so that real tickets were not closed without the owner.

## Owed before done

- AV-120 is still `doing`; this ticket's dep is not `closed`.

## Decisions taken without a ruling

- Started while AV-120 is `doing` (only the owner's usefulness check is owed); its code is committed in f32dd1c.
- The three lines are the count, `deeper look: /peep:goose health` and `next: clear to proceed`; the `loose:` line is dropped there (0 loose ends says it) except to carry the no-north-star nudge.
- `ducks/references/report.md` is kept as the `candidates.json` schema only, still at §2, because `goose/references/health.md` and `learned.rs` cite it; the page, engine and render steps went.
- map.mjs: only the `--report` path went. Its `index.html` page and CLI now have no skill caller either (Goose runs `report.mjs`), but they are graph code, not report code, so left for a later ticket.
- `map.test.mjs` keeps a `parsePatterns` check (six rows), since `report.mjs` still uses it; the rest of the report test went with the code.
- Ducks no longer drafts tickets for candidates or records `Considered, rejected` lines; the kb notes on those were removed.
