# peep

The Peep This skill pack: one Claude Code plugin, skills invoked as `/peep:<name>`. This folder is both the plugin and its local marketplace (`peep-this`).

## Local install

```
claude plugin marketplace add "<repo>/plugin"
claude plugin install peep@peep-this
```

Remove: `claude plugin uninstall peep@peep-this`, then `claude plugin marketplace remove peep-this`. Check: `claude plugin validate plugin`.

## Skills and floppy labels

Labels come from `design/skill-pack/FLOPPY-SPEC.md` (owner rulings 2026-11-15) and `design/skill-pack-2/SPEC.md`. Each skill's ticket copies its label into its `SKILL.md` frontmatter. Tweet has a floppy that seats on the plank but no tray cell.

| Skill | Command | Floppy label | Art id | Ticket |
|---|---|---|---|---|
| `ducks` | `/peep:ducks` | Ducks: get your ducks in a row | `ducks` | AV-58 |
| `qt` | `/peep:qt` | Question Time with Peep | `question-time` | AV-59 |
| `ticket` | `/peep:ticket` | Ticket | `ticket` | AV-57 |
| `packup` | `/peep:packup` | Pack Up | `handoff` | AV-60 |
| `shoot` | `/peep:shoot` | Shoot | `shoot` | AV-60 |
| `coderot` | `/peep:coderot` | Coderot | `coderot` | AV-61 |
| `walk` | `/peep:walk` | Walk Through | `walk` | AV-77 |
| `sneak` | `/peep:sneak` | Sneak Peep | `sneak` | AV-85 |
| `goose` | `/peep:goose` | Duck Duck Goose | `goose` | AV-92 |
| `bigpeep` | `/peep:bigpeep` | Big Peep | `bigpeep` | AV-90 |
| `tweet` | `/peep:tweet` | Tweet Tweet | `tweet` | AV-91 |

Goose makes report pages under `.peep/graph/`: `/peep:goose` (code map), `/peep:goose health` (hot spots, biggest files, untested areas, stale tickets, pattern cards; it also writes the `candidates.json` the Learned list reads) and `/peep:goose <topic>`. It took over the health and pattern report from Ducks (AV-120).

## Preamble (AV-117)

Every skill cites `references/preamble.md` once and keeps only its own `Why:` line. The preamble reads `.peep/setup.md`, applies `skills.off`, and bumps the run count in `.peep/runs`, showing the Why line while the old count is below 3. With no setup, the skill guesses one from the repo, shows a 5-line read-back, writes it on yes and carries on; Walk is not needed first. Pack Up and Tweet run without a setup.

## Context budget (every pack skill)

- `SKILL.md` under about 80 lines.
- Detail lives in the skill's `references/` and is read only when that step needs it.
- Output is a fixed terse format, defined in the skill; no free-form recaps.
- Shared formats (repo setup `.peep/setup.md`, ticket, area note, read-back) are in `references/formats.md`; skills cite its sections instead of restating them.
