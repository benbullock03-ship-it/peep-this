---
id: AV-120
title: Goose as report maker
status: open
depends: [AV-117]
---

# AV-120 — Goose as report maker

**What to build:** Goose's main job becomes a report page for the owner; agent querying is secondary (Question Time 2026-11-16).

- [ ] Three reports: `/peep:goose` (code map, as today), `/peep:goose health` (hot spots from git history, biggest files, untested areas, stale tickets), `/peep:goose <topic>` (a one-off page answering the owner's question from the graph)
- [ ] A read-back of what the report will cover before building; "yes" builds
- [ ] Repo health takes over what Ducks depth levels 2 to 5 produce: hot spots, pattern cards and `.peep/graph/candidates.json` in its current shape, so the Learned list (AV-110) reads it unchanged
- [ ] Pages written under `.peep/graph/`, one file each, art inlined, no network. Interim layout follows `design/skill-pack-2/graph.html`; AV-126 supplies the final template
- [ ] graphify offers kept as they are

**Done when:** each of the three commands writes and opens its page in this repo, `candidates.json` still feeds the Learned list, and the owner confirms the pages are useful.
