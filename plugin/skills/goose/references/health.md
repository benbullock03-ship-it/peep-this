# Health report

The page `/peep:goose health` writes. It takes over what Ducks depth levels 2 to 5 produce (`ducks/references/depth.md`): hot spots, candidate cards, pattern cards. Read-only; nothing edits code.

## 1. Gather

Four groups, always in this order, each with the rows it finds (empty rows show as "clear"):

- **Hot spots**: `git log --since=30.days --name-only --format=`, count per file; rows for files changed more than 10 times, top 5. `num` `"<n> changes"`, `small` how many tickets touched it if the log names them, else `"in 30 days"`.
- **Biggest files**: source files over 800 lines, top 5. `num` `"<n> lines"`, `small` `"over the 800-line mark"`.
- **Untested areas**: each `area.<name>` with no test file (`*.test.*`, `*_test.*`, `tests/`, a Rust `#[cfg(test)]`). `path` the area, `num` `"0 of <files>"`, `small` what you looked for.
- **Stale tickets**: tickets `doing` or `blocked` whose file has sat unchanged for more than the setup's `stale.days` (default 3). `path` `"<id> <title>"`, `tk` `"doing · 12 days"` or `"blocked · 9 days"` (set `blocked: true` for the second), `small` why it looks stale.

For hot spots and biggest files, `bar` is 1 to 10, the row's value against the largest in its card.

## 2. Candidates

Read the hot areas' most-touched files (10 at most) and each area note. A candidate passes the deletion test: removing it would concentrate complexity. Skip any on a `Considered, rejected:` line in the area note. Run one fresh-context subagent (Agent tool, general-purpose) with the repo root, the areas and this brief: "Find shallow modules, a concept spread across many files, leaks across module seams, and untested code. Report each as files plus one line; no fixes, no edits." Its findings go through the same test. Back each card's files with callers from `.peep/graph/graph.json` (run the map first). At most 5 cards, strongest first.

## 3. Write the data file

Anywhere outside `.peep/graph/` that you clean up, or a scratch path:

```
{ "lead": "<one sentence: how many findings, where to start>",
  "peep": "thinking" | "success",
  "foot": ["hot spots and stale tickets look back 30 days"],
  "cards": [ { "title": "Hot spots", "clear": "<what was checked, if no rows>",
               "rows": [ { "path": "...", "num": "...", "small": "...", "bar": 0, "tk": "", "blocked": false } ] }, ... ],
  "candidates": <the candidates JSON, schema in ${CLAUDE_PLUGIN_ROOT}/skills/ducks/references/report.md §2> }
```

`peep` is `success` only when every group is clear. `report.mjs` writes `.peep/graph/candidates.json` from `candidates` unchanged (the Learned list reads `cards[].pattern` from it) and turns each distinct `pattern` into a pattern card on the page from `${CLAUDE_PLUGIN_ROOT}/references/patterns.md`. A repo with no candidates still gets its findings; it shows no pattern card.

Then `report.mjs health .`, open the page, and name the top pick in the read-back. Drafting a ticket for it is Ducks' job.
