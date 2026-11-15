---
id: AV-53
title: Record the companion + skill pack vision in ADRs, CLAUDE.md and spec
status: closed
depends: []
attended: false
---

# AV-53 — Record the companion + skill pack vision

**What to build:** Write down the decisions from the domain-modelling session on 2026-11-15 (glossary in `CONTEXT.md`) so later tickets build against them.

- [x] `docs/adr/0001-widget-plus-skill-pack.md`: Peep This is the widget plus a `peep` skill pack shipped as one Claude Code plugin, for new Claude Code users. The widget shows and Jumps (copies the command, focuses the terminal) and never types into or starts sessions
- [x] `docs/adr/0002-clash-as-ambient-mood.md`: overturns the 2026-11-14 "not taken: collision radar" as an ambient clash mood only (no warning, no blocking, no trail entry)
- [x] `CLAUDE.md`: update "What this is", "Actions" and the Mission control "Not taken" line; add a short "Skill pack" section pointing to `CONTEXT.md` and the build order (pack, widget, moods)
- [x] `docs/spec.md`: new section for the skill pack, floppies, ticket shelf, moods and clash, using `CONTEXT.md` terms

**Done when:** the ADRs exist and `CLAUDE.md` and the spec no longer contradict `CONTEXT.md`.

## Decisions taken without a ruling

- `design/skill-pack/FLOPPY-SPEC.md` ruling 2 makes clash an attention state that pins both rows to the top and fills the bubble. That contradicts `CONTEXT.md` (a mood never changes state; clash is ambient). ADR 0002 and spec §13.6 keep the bolt tag and bracket but drop pinning, re-sort and attention fill. The design file is unchanged.
- Tired thresholds: AV-69 says sweat from 70%, yawn from 90%; AV-70 and the design put the nudge (yawn) at a setting, default 80, again at 95%. Spec §13.5 follows the later owner ruling (yawn and nudge at the threshold) and leaves the sweat point to AV-69.
- `CLAUDE.md` said a ticket archives at `done`; changed to `closed`, per `CONTEXT.md`.
- Dropped "desktop pet" from `CLAUDE.md` "What this is" and spec §12.0 (glossary avoids "pet").
- Skill pack section in `CLAUDE.md` and spec §13 are marked planned, matching how §11 and §12 are recorded.
