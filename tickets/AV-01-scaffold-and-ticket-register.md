---
id: AV-01
title: Tauri scaffold, design vendored, ticket register
status: in-progress
depends: []
attended: false
---

# AV-01 — Tauri scaffold, design vendored, ticket register

**What to build:** A Tauri 2 + Vite + TypeScript app that opens one window showing a single idle peep, with `design/peep.js` vendored unchanged into `src/vendor/`, Vitest wired, the spec in `docs/spec.md` and this ticket register with a generated index.

- [x] `git init`
- [x] Tauri 2 vanilla-ts scaffold, demo content and opener plugin removed
- [x] `src/vendor/peep.js` byte-identical to `design/peep.js`
- [x] `npm run build`, `npm test`, `cargo check` pass
- [x] `docs/spec.md`, `tickets/`, `npm run tickets`
- [x] `CLAUDE.md` commands section
- [ ] First commit

**Done when:** the above is committed and `npm run tauri dev` shows an idle peep.

## Owed before done

The commit. Not made without the owner's go.
