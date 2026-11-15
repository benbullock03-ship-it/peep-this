---
id: AV-85
title: Sneak Peep skill - 2 or 3 UI variants, pick one
status: doing
depends: [AV-74]
---

# AV-85 — Sneak Peep skill

**What to build:** `/peep:sneak` puts a UI design in front of the owner before it touches the app.

- [ ] Asks pointed design questions first (Question Time round rules), or takes them from a Question Time or Scrubadubdub run
- [ ] Builds 2 or 3 throwaway variants in `.peep/sneak/<slug>/`, served side by side on a local server; prints the URL
- [ ] The owner picks one: it moves to `design/<slug>/` as the reference; the rest are deleted
- [ ] Nothing in the app changes until a later ticket builds from the reference
- [ ] `.peep/sneak/` is gitignored by the skill
- [ ] Why line per AV-74

**Done when:** `/peep:sneak` on a test repo serves three variants, and picking one leaves only `design/<slug>/`.

## Decisions taken without a ruling

- Variants are named `a`, `b`, `c`, each a self-contained `index.html` (inline CSS/JS, no build, no network).
- Side by side means one overview page with an iframe per variant; each also at `/<variant>/`.
- Server is a dependency-free node script, `plugin/skills/sneak/scripts/serve.mjs`, bound to 127.0.0.1, random port unless given.
- The skill adds `.peep/sneak/` to `.gitignore` itself (the one edit outside `.peep/sneak/`).
- "None of these" deletes the sneak folder and writes nothing; "Change something first" rebuilds and serves again.
- Read-back (formats section 4) added before building, since the skill writes files.

## Owed before done

- A live `/peep:sneak` run on a test repo with the owner picking a variant. The scripted parts (serve, pick, gitignore) were verified by hand; the rounds and the AskUserQuestion pick were not run.

## Notes

- Verified in a throwaway repo: three variants served (overview had 3 iframes, `/b/` returned its page), simulated pick left only `design/demo/index.html`, `.peep/sneak/` matched `.gitignore`. `npx vitest run plugin/skills/sneak` passes.
- Owed: a live `/peep:sneak` run in a real session (the skill's question rounds and variant authoring are model behaviour, not scripted). The skill is not registered in `plugin/README.md` or `kb/plugin.md` (out of this worker's scope).
