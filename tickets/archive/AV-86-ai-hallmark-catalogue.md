---
id: AV-86
title: AI hallmark catalogue, scrub exceptions and fixture tests
status: closed
depends: []
---

# AV-86 — AI hallmark catalogue

**What to build:** The shared checklist Scrubadubdub (AV-87) reads, plus the per-repo exceptions format.

- [x] `plugin/references/ai-tells.md`: one entry per hallmark as `ID · severity · grep|eye · signal or regex · where it comes from · 2 alternatives`, grouped Colour, Type, Layout, Components, Motion, Copy, Code
- [x] Severity by who notices: P0 anyone, P1 a developer or designer, P2 craft only
- [x] Seed sources: the owner's `C:/Projects/datamapuk/.claude/skills/design-tropes/` (28 tropes, earned test), `C:/Projects/datamapuk/DESIGN.md`, `C:/Projects/datamapuk/.impeccable/critique/`; avoid-ai-design's catalogue (MIT, github.com/funboy322/avoid-ai-design), impeccable.style/slop, paralect.com/blog/signs-of-ai-slop-in-design, github.com/AdrianKrebs/design-slop-cop. Credit each borrowed entry
- [x] A "what not to over-flag" section: one hit is a nit; never claim a model made the page
- [x] `.peep/scrub.md` format in `formats.md`: `earned` (ID, path, reason; permanent) and `debt` (ID, path, count; may only shrink), plus inline `scrub-ignore: ID reason`
- [x] Anything declared in the repo's DESIGN.md is never flagged
- [x] Fixture tests: one bad and one good snippet per grep rule; a false positive is a bug
- [x] Seed this repo's `.peep/scrub.md` with AV-44's earned choices: 6px hard shadow, striped title bar, mono session names

**Done when:** the fixture tests pass and every grep rule has a failing and a passing snippet.

## Research (2026-11-15)

- Impeccable (Apache-2.0, github.com/pbakaus/impeccable): 62 deterministic rules, `npx impeccable detect --json`, exit 2 on findings, reason-carrying ignores. Its rules would flag AV-44's own shell, hence the seeded exceptions.
- Hallmarks beyond the owner's list: gradient headline text, Inter/Geist as the only face, the "tasteful" set (Space Grotesk, Instrument Serif, Fraunces), cream with `#D97757`, pill badge over the H1, "10K+ · 99.9% · 4.9★" stat strip, all-caps eyebrows, fade-up on every section, count-up numbers, icons in tinted tiles, arrows on every link, "Elevate/Seamless" copy.
- Unverified: avoid-ai-design's `detect.mjs` source; licences read from READMEs, not LICENSE files.
- 2026-11-15 build: `detect.mjs` read in full; licences confirmed from each repo's `LICENSE` via the GitHub API (aad MIT, slop-cop MIT, Impeccable Apache-2.0). All three owner sources were present.

## Built

- `plugin/references/ai-tells.md`: 82 entries, 47 grep and 35 eye. Grep by group: Colour 6, Type 6, Layout 4, Components 12, Motion 5, Copy 8, Code 6.
- `plugin/references/ai-tells/rules.mjs`: the grep rules plus `scan(text)` (line numbers, `min` thresholds, inline `scrub-ignore`). `rules.test.mjs`: a bad and a good snippet per rule, the good one must hit no rule at all; also checks the md's grep entries match `RULES` both ways with the same severity.
- `formats.md` §6 and `.peep/scrub.md` seeded.
- Scanning `src/styles.css` today also hits `tracked-caps` (7, the AV-44 chips and title), `side-stripe` (1), `ambient-loop` (2, emote frames and the waiting pulse), `hover-lift` (1) and `colour-literal` (6). Left for Scrubadubdub's first run to put to the owner.

## Decisions taken without a ruling

- IDs are lowercase slugs (`hard-shadow`), not `C1`-style codes, so `scrub-ignore: hard-shadow reason` reads on its own.
- Rule data lives in `plugin/references/ai-tells/` (`rules.mjs`, `rules.test.mjs`); fixtures are inline in the test, not separate files. The md holds signals, `rules.mjs` holds the regexes; the test keeps them in step.
- Each entry ends with a credit tag (`[aad C1]`, `[imp …]`, `[slopcop]`, `[paralect N]`, `[tropes N]`, `[crit]`, `[own]`). The owner's files are credited as "the owner's design-tropes skill" and "critique reports", with no project name.
- `earned` covers every hit of the ID under a path (file or folder prefix), not a count as in the owner's `tropeEarned.json`; `debt` carries the count.
- An inline `scrub-ignore` reason must start with a letter, so a bare ID before `*/` or `-->` waives nothing.
- DESIGN.md parsing is AV-87's job; this ticket states the rule in `ai-tells.md` and `formats.md` only.
- The seed earns only the three AV-44 choices the ticket names. Mono session names map to the eye rule `excessive-mono`. The bubble tails (`-4px 4px 0`) are hard shadows too, so the `hard-shadow` reason names them.
- `formats.md` intro changed from "five formats" to "six formats" to stay true.
- Severity: kept each source's level where one existed; `glass` stays P1 (aad), and the over-flag section notes that it is over-blamed.
