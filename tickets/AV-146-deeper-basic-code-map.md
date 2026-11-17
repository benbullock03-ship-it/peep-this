---
id: AV-146
title: Deeper code map, built-in and from graphify
status: open
depends: []
---

# AV-146: Deeper built-in code map

**What to build:** without graphify, `/peep:goose` draws areas from the setup's `area.*` prefixes. On this repo that gives "4 areas, 0 links" (2026-11-17): src/, src-tauri/ and plugin/ never import each other, so every card reads "used by nothing". The owner wants the map closer to graphify's depth.

- [ ] Areas finer than the setup's: split each area into its sub-folders or modules (e.g. `src-tauri/src/source/claude`, `ingest`, top-level Rust modules; frontend files grouped by feature) when an area holds more than about 15 files
- [ ] Cross-language links: a frontend `invoke("<cmd>")` links to the Rust `#[tauri::command] fn <cmd>`, and an event `emit`/`listen` pair links its two ends
- [ ] Rust `mod`/`use crate::` edges count as imports
- [ ] Clicking an area drills into its files and their links; the page stays offline and single-file
- [ ] With graphify: use its communities as areas (named from their main file or concept) instead of the setup's three prefixes, and drop `INFERRED`/`AMBIGUOUS` edges below a confidence floor. A 2026-11-17 build (2,953 nodes, 110 communities) gave a map of "4 areas, 4 links", about half of them false name collisions between minified identifiers (`t()`, `c()`, `restore()`), because `report.mjs`/`fromGraphify` keep only `source_file`
- [ ] `map.test.mjs` covers the invoke-to-command link, the sub-area split and the graphify community areas

**Done when:** `/peep:goose` without graphify on this repo shows more than 4 areas with links between frontend and backend, and drilling into an area lists its files.
