---
id: AV-105
title: Ducks level 5 - architecture report on the goose page
status: doing
depends: [AV-104]
---

# AV-105 — Ducks level 5: architecture report on the goose page

**What to build:** Level 5 renders the level 3-4 candidates as a page the owner looks at and learns from. It extends the goose page (`.peep/graph/index.html`): one offline file, peeps inlined, no CDN.

- [x] `map.mjs` (or a sibling script) takes a candidates JSON and adds a report section below the map
- [x] Legend, then one card per candidate: badge, files, before/after mini-diagram (hand-built SVG of modules and seams), one-line problem and fix, wins of six words or fewer, an "Explain like I'm 5" fold, an ADR-conflict callout when one applies
- [x] One top-pick card, the house peep beside it
- [x] Engine: goose's basic import map; graphify used when `graphify-out/graph.json` exists; install offered once, never run
- [x] Ducks opens the page, then asks which candidate to explore
- [x] Pixel-desktop look from `design/shell/`; Shoot both themes

**Done when:** `/peep:ducks` at level 5 on this repo opens the page with the map, at least one card with a working ELI5 fold, a top pick, and no network requests; the owner confirms what they saw.

## Owner rulings (grill 2026-11-16)

- Report first, map below. Top pick first, beside the 96px house peep.
- Each card: a peep whose state shows its strength (Strong waiting, Worth exploring thinking, Speculative idle), a pattern name and lesson from `plugin/references/patterns.md` (6 fixed entries), a stuck/working/success comic strip with the before and after diagrams, an ELI5 fold with its source.
- "Copy fix brief" button: a plain brief (candidate, files, deletion-test reason, target shape, steps, the setup's checks, what not to touch).
- Patterns feed the learned list: line added to AV-110.

## Decisions taken without a ruling

- Woken before AV-104 closed, on the owner's "finish 105".
- Candidates JSON is `.peep/graph/candidates.json` (generated); `map.mjs --report <file>` renders it. Its schema is in `ducks/references/report.md`.
- The graphify install offer is recorded once as `ducks.graphify: offered` in `.peep/runs`.

## Owed before done

- ~~Owner looks at `.peep/graph/index.html` and confirms what they saw~~: confirmed 2026-11-16 ("looks good I like it").
- `/peep:ducks` level 5 run end to end in a fresh session (this needs AV-104's ladder live run).
- Checked so far: map.test.mjs report test; Shoot dark at 1280 and light at 375. The report renders above the map, top pick first, 13 peeps upgraded, the ELI5 fold opens, copy shows its status, no horizontal scroll. The only request is the local server's favicon.ico.
