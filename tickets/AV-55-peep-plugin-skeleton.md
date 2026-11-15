---
id: AV-55
title: Peep plugin skeleton with a local marketplace
status: done
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

## Notes (2026-11-15)

- Built `plugin/.claude-plugin/{plugin.json,marketplace.json}`, `plugin/skills/hello/SKILL.md`, `plugin/README.md` (labels table, context budget rule).
- `claude plugin validate plugin`: passed with warnings (no author; directory-only: no LICENSE, no icon).
- Install test (CLI 2.1.325): `claude plugin marketplace add "<repo>/plugin"` then `claude plugin install peep@peep-this` (scope: user); `claude -p "/peep:hello"` from an empty scratch dir printed `peep: hello, the skill pack is installed.`
- Then `claude plugin uninstall peep@peep-this` and `claude plugin marketplace remove peep-this`; `settings.json`, `known_marketplaces.json` and `installed_plugins.json` byte-identical to the pre-test backups. The orphaned `plugins/cache/peep-this` left behind was deleted by hand.

## Decisions taken without a ruling

- `plugin/` is both the plugin and its marketplace: one `.claude-plugin/` holds both manifests, entry `source: "./"`.
- Marketplace name `peep-this`, so the install id is `peep@peep-this`.
- Floppy labels live in the `plugin/README.md` table for now, not in frontmatter; each skill ticket copies its label in.
- No `author`, `license` or icon in `plugin.json` yet; only needed for a directory submission.
