---
id: AV-147
title: Health report recommendation cards with explainers and fix prompts
status: open
depends: [AV-120]
---

# AV-147: Health report recommendation cards with explainers and fix prompts

**What to build:** the owner wants `/peep:goose health` to make actionable recommendations the way Matt Pocock's `improve-codebase-architecture` does, kept in Peep's teaching voice (owner, 2026-11-17). Today a candidate card is files plus one line and a pattern card; nothing says what to do or how to ask Claude to do it.

- [ ] Each candidate card becomes a recommendation card: **Files**, **Problem**, **Solution** (plain English, no interface design yet), **Benefits** (locality, leverage, and how tests improve), a **before / after** diagram, and a strength badge `Strong`, `Worth exploring` or `Speculative`. Vocabulary from the `codebase-design` skill (module, interface, depth, seam, adapter, leverage, locality, deletion test); domain names from `CONTEXT.md`
- [ ] Each card carries its pattern's explainer in more depth: why it hurts in this repo (2 to 4 sentences tied to the card's files), the ELI5, and the source with chapter or paper, from `plugin/references/patterns.md` (extended with a longer "why" per pattern)
- [ ] Each card has a copy-and-paste prompt to fix it, with a copy button: a self-contained prompt naming the files, the problem, the target shape and the checks to run, ending in the repo's rhythm (e.g. "grill me first with /peep:qt, then write tickets"). Prompts never ask Claude to start editing without a read-back
- [ ] A **Top recommendation** section at the end: which card first and why
- [ ] ADR conflicts: a card contradicting an ADR in `docs/adr/` shows only when the friction warrants reopening it, with a callout naming the ADR
- [ ] Offline and single-file as now: diagrams are inline SVG or CSS, no Tailwind or Mermaid CDN (the pack's no-network rule)
- [ ] `candidates.json` keeps `cards[].pattern` so the Learned list still reads it; new fields are additive
- [ ] `report.test.mjs` covers the new card fields, the copy prompt and the offline check

## Rulings (grill 2026-11-17)

1. Fix prompt: names files, problem and target shape, then runs `/peep:qt` on it so the interface is decided before code; tickets follow. No direct-edit prompt.
2. Diagram: inline SVG module boxes and arrows in the report's pixel style; shallow modules thin, the deepened module one thick box. No library.
3. Cards: up to 5, strongest first.
4. Explainer: 2 to 4 sentences on why it hurts in these files, the pattern's ELI5 and its source with chapter, folded under "Explain like I'm 5" as on the Learned page.
5. Strength: `Strong` = in a 30-day hot spot, deletion test concentrates, and tests get a better surface; `Worth exploring` = two of three; `Speculative` = one.
6. Speculative cards sit after the Top recommendation, collapsed under "Long shots (n)".
7. Rejection: the prompt ends with "if we don't do it, add a `Considered, rejected: <why>` line to the area note"; health already skips those. No ADR offer.
8. Widget: no change; copy buttons live on the page only (AV-131 churn rule).

**Done when:** `/peep:goose health` on this repo shows recommendation cards with problem, solution, benefits, before/after, strength, explainer with source and a working copy prompt, and the owner confirms they are actionable.
