---
id: AV-125
title: Remove progressive unlocks
status: done
depends: []
---

# AV-125 — Remove progressive unlocks

**What to build:** Core and Extras only, no rules; supersedes AV-108 (Question Time 2026-11-16).

- [x] Remove `src/unlocks.ts`, its tests, and the locked floppy state from the tray; Extras always show and work
- [x] Remove the Unlocks section and locked block from `plugin/skills/walk/references/tour.md`; every Extra keeps its one-line "use when"
- [x] Shoot still sits in Extras when the setup has no `url` (it cannot run there); that is the one rule left
- [x] Remove the `demo` repo's locked Extras fixture if nothing else uses it
- [x] AV-108: add a note `Superseded by AV-125 on 2026-11-16` and leave its status for the owner
- [x] Earned cosmetics (AV-115) are untouched: accessories are still earned, skills are never locked
- [x] Update spec and `CONTEXT.md` (neither named progressive unlocks; nothing to change)

**Done when:** a fresh repo's tray shows every Extra enabled, the tour prints no `Locked:` block, and the full run passes.

## Checks

1. `src/unlocks.ts` and `unlocks.test.ts` gone, no `./unlocks` import left: pass (files deleted, `npm run build` clean).
2. Tray has no locked state: pass (`tray.ts`, `rows.ts`, `styles.css` lock code removed; `npm test` 367 pass).
3. `tour.md` has no Unlocks section or `Locked:` block, Extras keep use-when, Shoot-without-`url` the one rule; `walk/SKILL.md` matches: pass (files read back).
4. Demo `unlock` fixture removed: pass (`src/demo.ts`).
5. AV-108 note added, status untouched: pass (`tickets/AV-108-progressive-unlocks.md`).
6. Earned cosmetics untouched: pass (`identity.ts`, `look.test.ts` not edited).
7. Spec and `CONTEXT.md` free of unlocks: pass (grep; only screen-lock and blocked-status hits).
8. Rust `Unlock`, its helpers and test removed: pass (`cargo check` clean, no warnings).
9. Full run: pass (`npm run build`; `npm test` 367; `cargo test` 422 + 17).
10. `?demo` tray in both themes shows no dimmed floppies: pass (Big Peep shot `.playwright-mcp/av125-tray-{dark,light}.png`: six floppies at opacity 1 plus `+5`, 0 `.is-locked`/`.tray-card-lock`, no "Locked" text).

## Decisions taken without a ruling

- The widget tray has no Core/Extras split, so the Shoot-without-`url` rule lives only in the tour; the tray shows every floppy alike.
- Removed the whole Rust `Unlock` (`Repo.unlock`, `run_count`, UI-area helpers and their tests) as well as the TS side, since nothing else reads them.
- Shoot without a `url` prints first in the Extras block with its use-when line.
- `kb/frontend.md` and `kb/plugin.md` were not in scope; they name no unlock rules.

## Done 2026-11-16

Checks 1 to 10 pass. `npm run build`, `npm test` (367) and `cargo test` (422 + 17) pass, and the `?demo` tray shots are clean in both themes.
