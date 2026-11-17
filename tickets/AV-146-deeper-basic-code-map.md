---
id: AV-146
title: Concept map for people and agents, from graphify or built-in
status: open
depends: []
---

# AV-146: Concept map for people and agents, from graphify or built-in

**What to build:** `/peep:goose`'s map shows "4 areas, 0 links" on this repo without graphify, and "4 areas, 4 links" (about half false) with it (2026-11-17: 2,953 nodes, 110 communities, but `report.mjs`/`fromGraphify` keep only `source_file`, and the false links are name collisions between minified identifiers like `t()`, `c()`, `restore()`). The owner wants the map as deep as graphify and readable both by them and by an AI model.

- [ ] Concepts, not setup areas: with graphify, merge its communities into about 8 to 12 named concepts in `CONTEXT.md` terms (e.g. State engine, Approvals, Rows); without it, build concepts from finer built-in areas (sub-folders or modules of an area over about 15 files)
- [ ] Naming: on each graphify build (or first run without it), the model groups and names concepts once, saved to `.peep/graph/concepts.json`; later runs reuse it; the owner renames by editing that file
- [ ] Edges: only graphify `EXTRACTED` calls, imports and references draw links; `INFERRED`/`AMBIGUOUS` edges hide behind a "show guesses" toggle
- [ ] Cross-language: a Goose pass links frontend `invoke("<cmd>")` to the Rust `#[tauri::command] fn <cmd>` and `emit`/`listen` event pairs, merged as extracted edges; Rust `mod`/`use crate::` count as imports in the built-in map
- [ ] Card: a plain one-line purpose, its 3 key files, entry points (Tauri commands, hook routes, skills) and needs/used-by counts
- [ ] Click: the pinned panel lists the concept's files, its strongest named calls in and out, and the concepts it touches; one offline single-file page
- [ ] For agents: `.peep/graph/map.md` lists each concept with the card's fields and its links; `kb/INDEX.md` points agents at it and at `graphify query` for detail
- [ ] Without graphify: the same page and `map.md` from built-in data, with one line offering graphify for more depth
- [ ] `map.test.mjs` covers the invoke-to-command link, concept grouping from communities and from built-in areas, the guess toggle, and `map.md`

**Done when:** `/peep:goose` on this repo shows 8 to 12 named concepts with links between frontend and backend and no false collision links, with and without graphify; clicking a concept lists its files; `.peep/graph/map.md` exists and an agent can answer "where is approval handled" from it alone.

## Rulings (grill 2026-11-17)

1. Areas: concepts merged from graphify communities, named in `CONTEXT.md` terms, about 8 to 12.
2. For AI: `map.html` for the owner plus a `.peep/graph/map.md` digest; `kb/INDEX.md` points at it; `graphify query` for detail.
3. Edges: `EXTRACTED` only; inferred behind a "show guesses" toggle.
4. Cross-language: Goose adds `invoke`-to-command and `emit`/`listen` edges.
5. Naming: model names once per graphify build, cached in `.peep/graph/concepts.json`, renamable by the owner.
6. Card: one-line purpose, 3 key files, entry points, needs/used-by counts.
7. Click: pinned panel with files, strongest named calls in and out, concepts touched; stays one offline page.
8. No graphify: same page and digest from built-in data, plus an offer line.
