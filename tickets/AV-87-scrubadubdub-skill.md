---
id: AV-87
title: Scrubadubdub skill - two passes, report, fix on yes
status: done
depends: [AV-74, AV-86]
---

# AV-87 — Scrubadubdub skill

**What to build:** `/peep:scrub` scrubs a UI of AI hallmarks and messy spacing and text, and teaches where each comes from.

- [x] Scope: changes since the last commit on main by default; `--full` for the whole repo
- [x] Pass 1, grep: the catalogue's grep rules (AV-86); when Impeccable is installed (`.impeccable/` or `npx impeccable`), run `impeccable detect --json` and merge its results into catalogue IDs
- [x] Pass 2, eye: judge Shoot screenshots at the setup's themes and widths; no `url` in setup: eye findings marked "inferred"
- [x] Honour `.peep/scrub.md` earned, debt and inline ignores, and DESIGN.md
- [x] Numbered report: file:line, ID, severity, where it comes from, 1 or 2 alternatives; findings appearing together score above single hits
- [x] Fix only the numbers the owner approves; every removal names what replaces it; re-scan and report the change
- [x] Offers `/peep:sneak` when an alternative needs a design choice
- [x] Why line per AV-74
- [x] Add the skill to Walk's Extras block with a use-when line

**Done when:** on a test page seeded with five hallmarks, `/peep:scrub` reports all five with alternatives and fixes only the approved ones.

## Decisions taken without a ruling

- Diff scope is changed lines (`git diff -U0 main`) plus whole untracked files, so old code in a touched file is not flagged; `--full` sees everything.
- Score: severity weight (P0 3, P1 2, P2 1) plus one per other ID in the same file; a single non-P0 hit is reported as P2 (catalogue "One hit is a nit").
- Debt at or under its count is suppressed and a lower count is reported to shrink; over the count, every hit is a finding.
- The scanner always skips the pack's own catalogue and seeded fixture; repo reference folders (`design/`) are passed as `--skip` by the skill, not hard-coded.
- Impeccable results map to catalogue IDs through the `imp <name>` credit tags; ones with no entry are listed unnumbered as `imp:<name>`. Not run here: Impeccable is not installed.
- Earned lines are offered, never written; debt lines only on `debt <n>`.
- Label `Scrubadubdub`, art id `scrub`, use-when line from `design/skill-pack-2/SPEC.md` §07. `ai-tells.md` now says `/peep:scrub` (was `/peep:scrubadubdub`).

## Done 2026-11-16

- `plugin/skills/scrub/SKILL.md`, scanner `plugin/skills/scrub/scripts/scan.mjs`, seeded page `plugin/skills/scrub/fixtures/seeded.html` (five hallmarks: default-face, purple-gradient, gradient-text, tracked-caps, glass).
- `scan.test.mjs`: all five found and clustered; a fix removes only that finding on re-scan; diff vs `--full` scope; P2 cap; earned and debt; diff parsing.
- Live run: a headless child `claude -p "/peep:scrub" --plugin-dir plugin` on a scratch repo holding the seeded page reported all five (plus one inferred eye finding), each with where it comes from and alternatives. Reply "1 2 4": fixed only 4, held back 1 and 2 as design choices and offered `/peep:sneak hero`, left 3 and 5 untouched; re-scan 5 to 4.
- Walk tour Extras line and README skills row added.
- Not run: the eye pass with a `url` (Shoot screenshots); the scratch repo had no dev server.
