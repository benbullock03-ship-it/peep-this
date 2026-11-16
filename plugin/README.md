# peep

The Peep This skill pack: one Claude Code plugin, skills invoked as `/peep:<name>`. This folder is both the plugin and its local marketplace (`peep-this`).

## Local install

```
claude plugin marketplace add "<repo>/plugin"
claude plugin install peep@peep-this
```

Remove: `claude plugin uninstall peep@peep-this`, then `claude plugin marketplace remove peep-this`. Check: `claude plugin validate plugin`.

## Skills and floppy labels

Labels come from `design/skill-pack/FLOPPY-SPEC.md` (owner rulings 2026-11-15). Each skill's ticket copies its label into its `SKILL.md` frontmatter.

| Skill | Command | Floppy label | Art id | Ticket |
|---|---|---|---|---|
| `ducks` | `/peep:ducks` | Ducks: get your ducks in a row | `ducks` | AV-58 |
| `qt` | `/peep:qt` | Question Time with Peep | `question-time` | AV-59 |
| `ticket` | `/peep:ticket` | Ticket | `ticket` | AV-57 |
| `packup` | `/peep:packup` | Pack Up | `handoff` | AV-60 |
| `shoot` | `/peep:shoot` | Shoot | `shoot` | AV-60 |
| `coderot` | `/peep:coderot` | Coderot | `coderot` | AV-61 |
| `reel` | `/peep:reel` | none yet (AV-84) | none yet | AV-75 |
| `unpack` | `/peep:unpack` | none yet (AV-84) | none yet | AV-76 |
| `walk` | `/peep:walk` | none yet (AV-84) | none yet | AV-77 |
| `sneak` | `/peep:sneak` | none yet (AV-89) | none yet | AV-85 |
| `scrub` | `/peep:scrub` | Scrubadubdub | `scrub` | AV-87 |
| `goose` | `/peep:goose` | none yet (AV-94) | none yet | AV-92 |
| `bigpeep` | `/peep:bigpeep` | none yet (AV-94) | none yet | AV-90 |
| `tweet` | `/peep:tweet` | none yet (AV-94) | none yet | AV-91 |
| `hello` | `/peep:hello` | none (install check, AV-55) | none | AV-55 |

## Why line (AV-74)

Every skill, and every new one from AV-75 on, ends with a run-count step: bump its line in `.peep/runs` and, while the old count is below 3, close the reply with one `Why:` line tied to a habit (think before code, finish before starting, prove it works). Rule: `references/formats.md` §1.

## Context budget (every pack skill)

- `SKILL.md` under about 80 lines.
- Detail lives in the skill's `references/` and is read only when that step needs it.
- Output is a fixed terse format, defined in the skill; no free-form recaps.
- Shared formats (repo setup `.peep/setup.md`, ticket, area note, read-back) are in `references/formats.md`; skills cite its sections instead of restating them.
