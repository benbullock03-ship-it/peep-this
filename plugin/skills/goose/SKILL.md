---
name: goose
description: "Duck Duck Goose: a report page for the owner, three kinds: the code map, repo health, or a one-off answer to a question. Use when the user runs /peep:goose, /peep:goose health or /peep:goose <topic>."
disable-model-invocation: true
label: Duck Duck Goose
---

# /peep:goose

Finds the goose in the ducks: a report page the owner looks at and learns from. Agents query the map instead of grepping blind; that is the side job.
"formats" below is the pack's shared file `${CLAUDE_PLUGIN_ROOT}/references/formats.md`.

- `/peep:goose`: the code map. Areas as cards, who needs whom, live sessions' peeps on the areas they edit.
- `/peep:goose health`: hot spots, biggest files, untested areas, stale tickets, and pattern cards.
- `/peep:goose <topic>`: a one-off page that answers the owner's question from the graph.

## 1. Read the repo setup

Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count). Take the `area.<name>` lines. Files outside every area go in `other`.

## 2. Pick the engine

- `graphify` on PATH (`graphify --help` prints) and `graphify-out/graph.json` exists: use it. Refresh only when the owner says so (`/graphify . --update` per `~/.claude/skills/graphify/SKILL.md`; a full build calls a model, so say that first).
- Installed but no `graphify-out/`: offer `/graphify .`, which builds it. Meanwhile run the basic map below; do not wait.
- Not installed: offer `uv tool install graphifyy` (or `pip install graphifyy`), then `graphify install`. Never install without a yes. Meanwhile run the basic map.

## 3. Read back, then build

Say what the report will cover, at most 4 lines, and wait for the owner's yes (a correction changes the plan first):

- map: the engine, the areas, whether sessions are shown.
- health: the four groups, the look-back (`30 days`; stale tickets use the setup's `stale.days`), the depth (pattern cards, with a fresh-read sweep).
- topic: the question as you read it, and which areas and files you will read.

On a yes, build the report per its reference. Every page is one file in `.peep/graph/` with fonts and art inlined and no network; the three pages link to each other as tabs. Generated: if `.gitignore` lacks `.peep/graph/`, say so rather than editing it.

Run from the repo root; add `--graphify graphify-out/graph.json` when the engine is graphify:

- map: `node "${CLAUDE_PLUGIN_ROOT}/skills/goose/scripts/report.mjs" map .` writes `map.html` and `graph.json`. It reads `.ts .tsx .js .mjs .rs` imports, folds them into area links, and reads `~/.claude/sessions/*.json` and the transcript tails for who is editing what (last 30 minutes).
- health: gather and write the JSON per `references/health.md`, then `report.mjs health . --data <file>`.
- topic: run the map first so `graph.json` is fresh, answer per `references/topic.md`, then `report.mjs topic . --data <file>`.

Print the JSON line it returns, then open the page: `start "" ".peep/graph/<kind>.html"` (Windows), `open` (macOS) or `xdg-open`.

## 4. Tell agents how to query

After the first map, add to `kb/INDEX.md` (formats §3) a short section `## Querying the map` if it is not there, in these terms:

- Who calls `x`: edges in `.peep/graph/graph.json` are `[from, to]` pairs meaning from imports to; filter those whose `to` is the file.
- With graphify: `graphify query "what calls state.rs"`, `graphify path "A" "B"`, `graphify explain "X"`.
- Read the map before grepping; regenerate it with `/peep:goose` after big moves.

Use Edit; never rewrite the index.

## 5. Read back the result

At most 6 lines: the report, engine, the counts that matter (map: areas, links, sessions; health: findings per group, top pick; topic: files read), the page path. A "what calls X" question is answered from `graph.json`, naming the file list.

## 6. Last step: run count

Per the preamble: `Why: know what a change touches before making it, so you think before code.`
