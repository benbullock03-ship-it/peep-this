---
id: AV-55
title: Peep plugin skeleton with a local marketplace
status: open
depends: []
attended: false
---

# AV-55 — Peep plugin skeleton

**What to build:** An empty `peep` Claude Code plugin in this repo that installs locally and namespaces its skills as `/peep:<name>`.

- [ ] `plugin/` folder: `.claude-plugin/plugin.json` (name `peep`), `skills/` and a local `.claude-plugin/marketplace.json`. Check the current plugin layout in the Claude Code docs first; don't guess it
- [ ] One placeholder skill proves `/peep:hello` runs from any repo after a local install, and uninstalls cleanly
- [ ] Context budget rule for every pack skill, written in `plugin/README.md`: `SKILL.md` under about 80 lines, detail in `references/` read only when needed, outputs in a fixed terse format
- [ ] Short commands, cute labels: `ticket`, `ducks`, `qt`, `packup`, `shoot`, `coderot`; each skill's frontmatter or README carries its floppy label ("Question Time with Peep", "Get your ducks in a row", ...)

Decision taken for the owner to overrule: the plugin source lives in this repo under `plugin/` and ships with the widget release.

**Done when:** a local install gives `/peep:hello` in a fresh session in another repo.
