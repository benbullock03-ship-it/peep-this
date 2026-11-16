# Candidates JSON

Not a Ducks step: Ducks makes no cards or pages. `/peep:goose health` writes this file (`skills/goose/references/health.md`) and the widget's Learned list reads `cards[].pattern` from it. The schema stays here at §2 because health.md and `learned.rs` cite it.

## 1. Where

`.peep/graph/candidates.json`, written by `report.mjs health` from the data file's `candidates`. Generated, never committed.

## 2. Schema

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

Diagrams: 2 to 5 modules each, short names, edges are imports `[from, to]` by index, drawn from `.peep/graph/graph.json`, not invented.
