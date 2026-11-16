# Level 5: the report page

Level 4's cards, rendered as a page the owner looks at and learns from. It extends the goose page: one offline file, peeps inlined, no network.

## 1. Engine

`graphify-out/graph.json` exists: pass `--graphify graphify-out/graph.json`. Not there and `ducks.graphify` is missing from `.peep/runs`: offer the graphify install once, in one line (per goose `SKILL.md` §2), set `ducks.graphify: offered`, and never run it here. Otherwise the basic import map.

## 2. Candidates JSON

Write `.peep/graph/candidates.json` (generated, never committed):

```
{ "top": <index of the top pick>,
  "commands": { "build": "<setup build>", "test": "<setup test>" },
  "cards": [ {
    "name": "...", "badge": "Strong|Worth exploring|Speculative",
    "pattern": "<id from ${CLAUDE_PLUGIN_ROOT}/references/patterns.md>",
    "files": ["..."], "problem": "<one line>", "fix": "<one line>",
    "why": "<the deletion test, one line>", "target": "<the shape after>",
    "steps": ["...", "..."], "dont": "<what not to touch>",
    "wins": ["<six words or fewer>", "..."],
    "eli5": "<optional; the pattern's ELI5 is the fallback>",
    "adr": "<only when the fix contradicts an ADR: its number and why>",
    "step": "<a few words for the comic's middle panel>",
    "before": { "nodes": ["<module>", "..."], "edges": [[0, 1]] },
    "after":  { "nodes": ["..."], "edges": [[0, 1]] } } ] }
```

Diagrams: 2 to 5 modules each, short names (13 characters show), edges are imports `[from, to]` by index, drawn from the map's `graph.json`, not invented.

## 3. Render and open

`node "${CLAUDE_PLUGIN_ROOT}/skills/goose/scripts/map.mjs" . [--graphify ...] --report .peep/graph/candidates.json`, then open `.peep/graph/index.html` (goose `SKILL.md` §3). The page puts the candidates above the map: legend, the top pick beside the house peep, then each card with a peep showing its strength, the pattern lesson, a stuck, working, success comic strip, an ELI5 fold and a "Copy fix brief" button.

Then the same question as level 3: "Which candidate to explore?", the top pick recommended. A pick drafts a ticket on a yes; the brief on the card is that ticket's starting text.
