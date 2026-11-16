---
id: AV-120
title: Goose as report maker
status: doing
depends: [AV-117]
---

# AV-120 — Goose as report maker

**What to build:** Goose's main job becomes a report page for the owner; agent querying is secondary (Question Time 2026-11-16).

- [x] Three reports: `/peep:goose` (code map, as today), `/peep:goose health` (hot spots from git history, biggest files, untested areas, stale tickets), `/peep:goose <topic>` (a one-off page answering the owner's question from the graph)
- [x] A read-back of what the report will cover before building; "yes" builds
- [x] Repo health takes over what Ducks depth levels 2 to 5 produce: hot spots, pattern cards and `.peep/graph/candidates.json` in its current shape, so the Learned list (AV-110) reads it unchanged
- [x] Pages written under `.peep/graph/`, one file each, art inlined, no network. Interim layout follows `design/skill-pack-2/graph.html`; AV-126 supplies the final template
- [x] graphify offers kept as they are

**Done when:** each of the three commands writes and opens its page in this repo, `candidates.json` still feeds the Learned list, and the owner confirms the pages are useful.

## Checks

1. pass: map writes `map.html` and `graph.json`. `report.test.mjs` "map writes graph.json..."; also run on a scratch repo, 4 areas and 5 links drawn in Playwright.
2. pass: health and topic write their pages. `report.test.mjs` health and topic tests; scratch runs loaded in Playwright (`.go-card`, `.go-pat`, `.go-q`, slice "3 of 4 areas.").
3. pass: offline, art inlined. Test asserts no `src`/`href` starting `http`; Playwright found no external URL on the topic page; fonts, peep.js and floppies are inlined in `assets/report.html`.
4. pass: read-back then "yes" builds, for all three. `plugin/skills/goose/SKILL.md` §3.
5. pass: health takes over Ducks levels 2 to 5. `references/health.md` §1 to §3; the test checks `candidates.json` keeps `cards[].pattern`, which `learned.rs` `cited_patterns` reads (`cargo test` passes).
6. pass: graphify offers unchanged. SKILL.md §2 is the old text.
7. pass: AV-126 final template. Built page has the `go-*` frame, title bar and tabs (screenshot of `map.html`).
8. pass: `npm run build`, `cargo check`, `npm test` (367), `cargo test` (422).
9. owed to the owner: the pages are useful.
10. owed to the owner: a run of each command in this repo (not done here: the session could not write `.peep/graph/` in this repo).

## Decisions taken without a ruling

- Page names `map.html`, `health.html`, `topic.html` in `.peep/graph/` (the tabs link to them); `index.html` and `map.mjs` stay for Ducks levels 4 and 5 until AV-121.
- `assets/report.html` is baked by `scripts/bake.mjs` from `design/goose-report/` (fonts and floppies are binary), with the renderer copy `scripts/report-client.js`; rerun bake when the design changes.
- The demo link router handled neighbouring columns only; `report-client.js` routes other links under the target's row (lanes capped at 8 rows).
- Pattern cards on the health page are derived from `candidates.json` cards and `patterns.md`; they carry no link, since `patterns.md` sources are text.
- Health stale tickets use the setup's `stale.days` (default 3), not the design's 7 days; hot spots look back 30 days, threshold 10 changes.
- The health run names a top pick but does not ask "which candidate to explore"; that stays with Ducks (AV-121).
- The topic page's side map is computed by `report.mjs` from `slice.ids`, so the model only names areas.

## Owed before done

- Owner: run `/peep:goose`, `/peep:goose health` and `/peep:goose <topic>` in this repo and confirm the pages are useful.
- Big Peep: README, CONTEXT and tour wording (see report); `ducks/SKILL.md` depth 5 and `ducks/references/report.md` still point at the old page (AV-121).
