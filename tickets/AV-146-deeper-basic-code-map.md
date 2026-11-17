---
id: AV-146
title: Deeper built-in code map
status: open
depends: []
---

# AV-146: Deeper built-in code map

**What to build:** without graphify, `/peep:goose` draws areas from the setup's `area.*` prefixes. On this repo that gives "4 areas, 0 links" (2026-11-17): src/, src-tauri/ and plugin/ never import each other, so every card reads "used by nothing". The owner wants the map closer to graphify's depth.

- [ ] Areas finer than the setup's: split each area into its sub-folders or modules (e.g. `src-tauri/src/source/claude`, `ingest`, top-level Rust modules; frontend files grouped by feature) when an area holds more than about 15 files
- [ ] Cross-language links: a frontend `invoke("<cmd>")` links to the Rust `#[tauri::command] fn <cmd>`, and an event `emit`/`listen` pair links its two ends
- [ ] Rust `mod`/`use crate::` edges count as imports
- [ ] Clicking an area drills into its files and their links; the page stays offline and single-file
- [ ] `map.test.mjs` covers the invoke-to-command link and the sub-area split

**Done when:** `/peep:goose` without graphify on this repo shows more than 4 areas with links between frontend and backend, and drilling into an area lists its files.
