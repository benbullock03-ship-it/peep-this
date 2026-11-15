---
id: AV-92
title: Duck Duck Goose skill - dependency map and peep graph page
status: doing
depends: [AV-74]
---

# AV-92 — Duck Duck Goose skill

**What to build:** `/peep:goose` maps the codebase's dependencies so agents query it instead of searching aimlessly, and writes an interactive graph page for the owner.

- [ ] Uses graphify when installed (build and query its graph)
- [ ] Not installed: offers the install command, and meanwhile builds a basic import map plus the repo setup's areas
- [ ] Writes `.peep/graph/index.html`: standalone, offline, areas as clusters, live sessions' peeps standing on the areas they are editing; opens it in the browser
- [ ] Tells agents how to query the map (a short section in each area note or `kb/INDEX.md`)
- [ ] Why line per AV-74

**Done when:** `/peep:goose` on this repo, with and without graphify, produces a graph page that opens offline and an agent answers "what calls state.rs" from the map.

## Decisions taken without a ruling

- Query instructions live in `kb/INDEX.md` (one short section), not in each area note.
- Importer covers ts/js and Rust (`crate::`, `mod x;`) only; other languages need graphify.
- `design/peep.js` is copied byte-identical to `plugin/skills/goose/assets/peep.js` so the shipped plugin can inline it.
- A session shows on an area only when its cwd is inside that area's folder; a repo-root cwd gets a `repo root` card.
- Styling is token-based and plain until AV-93 art lands; `.peep/graph/` left to the owner to gitignore.

## Owed before done

- The graphify path: run `/graphify .` once (costs model calls), then `map.mjs . --graphify graphify-out/graph.json`; it has no test yet.
- Owner opens `.peep/graph/index.html` offline and confirms the page.
- Peeps are placed by session cwd, not by the files being edited; most sessions sit at the repo root and land on the `repo root` card. Placing by edited files needs hook file paths from the backend: owner to rule whether that is in scope here or a follow-up.

## Notes

- Built: `plugin/skills/goose/{SKILL.md,scripts/map.mjs,scripts/map.test.mjs,assets/peep.js}`.
- Basic path verified on this repo (117 files, 300 imports); the "what calls state.rs" answer matched `grep crate::state` plus `lib.rs` (`mod state;`).
- Owed: the graphify path (`--graphify`) is untested against a real `graphify-out/graph.json` (none built; a full build costs model calls). Owner walkthrough of the page in a browser.
