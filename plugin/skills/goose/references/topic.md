# Topic report

The page `/peep:goose <topic>` writes: the owner's question answered from the graph, as a page.

## 1. Answer

Read `.peep/graph/graph.json` (run `report.mjs map .` first if it is missing or older than the last big move), find the files the question touches by their edges, then read those files. Answer from what you read, not from memory; say so when you could not tell.

- A short answer is `short` plus one to three paragraphs with no headings. A long answer has headings numbered `1 · ...` and ends with a `Read first` heading and one paragraph.
- Prose is plain: no jargon the owner has not met. Backticks make inline code.
- `short` is one or two sentences and always present.

## 2. Write the data file

```
{ "question": "<the owner's question, as asked>",
  "asked": "asked by the owner · answered from <n> files in <m> areas",
  "short": "...",
  "answer": [ { "h": "1 · ..." }, { "p": "..." } ],
  "files": [ { "path": "...", "why": "<a few words>" } ],
  "slice": { "ids": ["<area>", "..."] },
  "foot": ["answered from the code at <short commit>; it may be stale after that"] }
```

`slice.ids` names the areas the answer drew on; `report.mjs` lays out their side map and writes "n of m areas." Then `report.mjs topic . --data <file>` and open the page.
