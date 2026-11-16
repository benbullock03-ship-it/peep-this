---
id: AV-85
title: Sneak Peep skill - 2 or 3 UI variants, pick one
status: closed
depends: [AV-74]
---

# AV-85 — Sneak Peep skill

**What to build:** `/peep:sneak` puts a UI design in front of the owner before it touches the app.

- [x] Asks pointed design questions first (Question Time round rules), or takes them from a Question Time or Scrubadubdub run
- [x] Builds 2 or 3 throwaway variants in `.peep/sneak/<slug>/`, served side by side on a local server; prints the URL
- [x] The owner picks one: it moves to `design/<slug>/` as the reference; the rest are deleted
- [x] Nothing in the app changes until a later ticket builds from the reference
- [x] `.peep/sneak/` is gitignored by the skill
- [x] Why line per AV-74

**Done when:** `/peep:sneak` on a test repo serves three variants, and picking one leaves only `design/<slug>/`.

## Decisions taken without a ruling

- Variants are named `a`, `b`, `c`, each a self-contained `index.html` (inline CSS/JS, no build, no network).
- Side by side means one overview page with an iframe per variant; each also at `/<variant>/`.
- Server is a dependency-free node script, `plugin/skills/sneak/scripts/serve.mjs`, bound to 127.0.0.1, random port unless given.
- The skill adds `.peep/sneak/` to `.gitignore` itself (the one edit outside `.peep/sneak/`).
- "None of these" deletes the sneak folder and writes nothing; "Change something first" rebuilds and serves again.
- Read-back (formats section 4) added before building, since the skill writes files.

## Notes

- Verified in a throwaway repo: three variants served (overview had 3 iframes, `/b/` returned its page), simulated pick left only `design/demo/index.html`, `.peep/sneak/` matched `.gitignore`. `npx vitest run plugin/skills/sneak` passes.
- Live run 2026-11-16 in `F:\Projects\sneak-test`: owner ran `/peep:sneak habit list home screen`, answered the rounds, picked a variant and confirmed it works. Afterwards only `design/habit-list-home/index.html` remained, `.peep/sneak/` was gone and gitignored, `src/` untouched, `.peep/runs` had `sneak: 1`, no server left running. Owner closed it.
- Registered in `plugin/README.md`, `kb/plugin.md` and Walk's tour (`walk/references/tour.md`).
