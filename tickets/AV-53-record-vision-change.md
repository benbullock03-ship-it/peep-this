---
id: AV-53
title: Record the companion + skill pack vision in ADRs, CLAUDE.md and spec
status: open
depends: []
attended: false
---

# AV-53 — Record the companion + skill pack vision

**What to build:** Write down the decisions from the domain-modelling session on 2026-11-15 (glossary in `CONTEXT.md`) so later tickets build against them.

- [ ] `docs/adr/0001-widget-plus-skill-pack.md`: Peep This is the widget plus a `peep` skill pack shipped as one Claude Code plugin, for new Claude Code users. The widget shows and Jumps (copies the command, focuses the terminal) and never types into or starts sessions
- [ ] `docs/adr/0002-clash-as-ambient-mood.md`: overturns the 2026-11-14 "not taken: collision radar" as an ambient clash mood only (no warning, no blocking, no trail entry)
- [ ] `CLAUDE.md`: update "What this is", "Actions" and the Mission control "Not taken" line; add a short "Skill pack" section pointing to `CONTEXT.md` and the build order (pack, widget, moods)
- [ ] `docs/spec.md`: new section for the skill pack, floppies, ticket shelf, moods and clash, using `CONTEXT.md` terms

**Done when:** the ADRs exist and `CLAUDE.md` and the spec no longer contradict `CONTEXT.md`.
