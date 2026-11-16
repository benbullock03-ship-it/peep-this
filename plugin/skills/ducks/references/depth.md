# Depth: how deep Ducks looks when nothing is loose

Asked only when nothing is loose. Levels stack: each does everything above it. The option's description states its rough cost before anything runs.

| Level | Option label | Cost (in the description) | Adds |
|---|---|---|---|
| 1 | `1 Today (recommended)` | nothing more | `clear to proceed` |
| 2 | `2 Hot spots` | one git log | hot spots |
| 3 | `3 Candidates` | a few minutes of reading the hot areas | candidate cards, one top pick |
| 4 | `4 Sweep` | a subagent's fresh read plus the goose map, several minutes and tokens | sweep findings as more cards |
| 5 | `5 Report` | level 4 plus a page to look at | the report page (`report.md`) |

AskUserQuestion takes at most 4 options: offer 1, 2, 3 and 4; the description of 4 says "or reply 5 for the page". An "Other" of `5` means level 5.

## 2: hot spots

`git log -20 --name-only --format=`, each file counted under the first `area.<name>` whose prefix matches (F§1), the rest under `other`. Print the top 3 as `hot: <area> <n> files` lines, most first.

## 3: candidate cards

Read the hot areas' most-touched files (top 10 at most) and the area note for each. A candidate is a module or seam where **deleting it would concentrate complexity** (the deletion test, the only filter): it earns its place, but in the wrong shape, e.g. a shallow module, a concept spread over many files, a leak across a seam. Something whose deletion would just remove complexity is a plain cleanup, not a candidate.

- Skip any candidate already named on a `Considered, rejected:` line in that area's note.
- One card per candidate, at most 5, strongest first:

```
<n>. [Strong|Worth exploring|Speculative] <name>
   files: <path>, <path>
   problem: <one line>
   fix: <one line>
```

- Then `top pick: <n>, <why in a few words>`, and stop: one AskUserQuestion, "Which candidate to explore?", the top pick first and recommended, plus "None for now".
- A pick: draft a ticket for it per F§2 on a yes. Never edit code.
- "None for now" or a candidate the owner rejects with a reason: add one line to the area note's `## Why it's like this`, `Considered, rejected: <what> - <why>`, by Edit, committed by name as `ducks: rejected <what>`. Write an ADR in `docs/adr/` as well only when the reason is hard to reverse or would surprise a reader.

## 4: sweep

Before the cards, two runs in parallel:

- One fresh-context subagent (Agent tool, general-purpose) given only the repo root, the setup's areas and this brief: "Find shallow modules (interface as complex as the body), a concept spread across many files, leaks across module seams, and untested code. Report each as files plus one line; no fixes, no edits." Its findings go through the same deletion test and skip list, then join the cards.
- The goose basic map: `node "${CLAUDE_PLUGIN_ROOT}/skills/goose/scripts/map.mjs" .`. Use its `graph.json` callers to back each card's files (a card's module with many callers across areas is stronger).
