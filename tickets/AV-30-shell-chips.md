---
id: AV-30
title: Shell chip rows
status: todo
depends: [AV-29, AV-28]
attended: false
---

# AV-30 — Shell chip rows

**What to build:** Stub. A chip row per background shell under the parent, below any child peeps (spec §11.7).

- [ ] Chip: `>_` glyph, command (mono, middle-truncated), live runtime, state label; `serving` shows the port
- [ ] No peep on shell rows
- [ ] `failed` puts a red dot on the parent row that stays visible when children are collapsed; hover shows exit code and last line
- [ ] `exited` chips linger 30s then fold, like finished subagents
- [ ] Vitest: formatting and fold timing

**Done when:** the owner saw a dev server chip showing its port and a failed build flagged on a collapsed parent.
