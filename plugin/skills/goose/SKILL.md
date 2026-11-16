---
name: goose
description: "Duck Duck Goose: map who calls whom on a graph page with live sessions' peeps on the areas they work in. Use when the user runs /peep:goose."
disable-model-invocation: true
label: Duck Duck Goose
---

# /peep:goose

Finds the goose in the ducks: which file calls which. Agents query the map instead of grepping blind; the owner gets a page to look at.
"formats" below is the pack's shared file `${CLAUDE_PLUGIN_ROOT}/references/formats.md`.

## 1. Read the repo setup

Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count). Take the `area.<name>` lines. The page still groups by area; files outside every area go in `other`.

## 2. Pick the engine

- `graphify` on PATH (`graphify --help` prints) and `graphify-out/graph.json` exists: use it. Refresh only when the owner says so (`/graphify . --update` per `~/.claude/skills/graphify/SKILL.md`; a full build calls a model, so say that first).
- Installed but no `graphify-out/`: offer `/graphify .`, which builds it. Meanwhile run the basic map below; do not wait.
- Not installed: offer `uv tool install graphifyy` (or `pip install graphifyy`), then `graphify install`. Never install without a yes. Meanwhile run the basic map.

## 3. Build the map and the page

Run from the repo root:

- Basic import map: `node "${CLAUDE_PLUGIN_ROOT}/skills/goose/scripts/map.mjs" .`
- From graphify: `node "${CLAUDE_PLUGIN_ROOT}/skills/goose/scripts/map.mjs" . --graphify graphify-out/graph.json`

It reads `.ts .tsx .js .mjs .rs` imports (relative imports; Rust `crate::` paths and `mod x;`), assigns each file to the first area whose prefix matches (formats §1), reads `~/.claude/sessions/*.json` for sessions whose `cwd` is inside the repo, and writes `.peep/graph/graph.json` and `.peep/graph/index.html`. A session's peep stands on every area its Edit and Write calls touched in the last 30 minutes (read from the tail of its transcript in `~/.claude/projects/`). With no recent edits it stands on the area its `cwd` is in, or on a `repo root` card. The page is one file with the art inlined: no network.

Print the JSON line it returns. Then open the page: `start "" ".peep/graph/index.html"` (Windows), `open` (macOS) or `xdg-open`. `.peep/graph/` is generated; if `.gitignore` lacks it, say so rather than editing it.

## 4. Tell agents how to query

Add to `kb/INDEX.md` (formats §3) a short section `## Querying the map` if it is not there, in these terms:

- Who calls `x`: edges in `.peep/graph/graph.json` are `[from, to]` pairs meaning from imports to; filter those whose `to` is the file.
- With graphify: `graphify query "what calls state.rs"`, `graphify path "A" "B"`, `graphify explain "X"`.
- Read the map before grepping; regenerate it with `/peep:goose` after big moves.

Use Edit; never rewrite the index.

## 5. Read back

At most 6 lines: engine used, files, imports, areas, sessions shown, page path. If asked a "what calls X" question, answer it from `graph.json`, naming the file list.

## 6. Last step: run count

Per the preamble: `Why: know what a change touches before making it, so you think before code.`
