# Areas

- [frontend](frontend.md): widget UI in `src/`: rows, open row, tray, ticket shelf, Jump
- [backend](backend.md): Rust core in `src-tauri/`: state engine, ingest, store, moods
- [plugin](plugin.md): the `peep` skill pack in `plugin/`: skills, shared formats, run counts

## Querying the map

- `/peep:goose` writes `.peep/graph/graph.json` (`edges` are `[from, to]`: from imports to) and `map.html`. Who calls a file: filter edges by `to`.
- With graphify built: `graphify query "what calls state.rs"`, `graphify path "A" "B"`, `graphify explain "X"`.
- Read the map before grepping; rerun `/peep:goose` after big moves.
