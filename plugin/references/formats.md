# Pack formats

The five formats every Peep skill reads or writes. Cite the section; do not restate it. Examples are in `examples/`.

All frontmatter is flat: one `key: value` per line, no nesting, no multi-line values. Lists are `[a, b]` or comma-separated. A missing key takes its default.

## 1. Repo setup: `.peep/setup.md`

At the repo root. Walk (`/peep:walk`) writes it on first run; the owner may hand-edit it. Every Peep skill reads it instead of guessing. The `.peep/` folder also holds `handoff.md`, Pack Up's note (AV-60).

| Key | Meaning | Default |
|---|---|---|
| `prefix` | Ticket id prefix, e.g. `AV` | none; Walk asks |
| `tickets` | Tickets folder, relative to the repo root | `tickets/` |
| `archive` | Where `closed` tickets move | `<tickets>archive/` |
| `index` | Command that regenerates a ticket index after a status change | none (skip) |
| `build` | Build or type-check command | none |
| `test` | Test command; a failure makes the peep worried | none |
| `dev` | Command that starts the dev server, for Shoot | none |
| `url` | Dev URL Shoot opens | none (no Shoot) |
| `themes` | Themes Shoot covers | `light, dark` |
| `widths` | Viewport widths in px Shoot covers | `375, 1280` |
| `area.<name>` | One area: comma-separated path prefixes | none |
| `qt.round` | Questions per Question Time round, 2 to 4 | `4` |
| `packup.questions` | `ask` (ask when the next step is unclear) or `never` (write straight away) | `ask` |
| `shoot.widths` | Widths Shoot covers; wins over `widths` | `widths` |
| `skills.off` | Pack skills to switch off, comma-separated names (`coderot, reel`) | none |

- `skills.off`: a skill listed there replies `peep <name>: switched off in .peep/setup.md.` and stops, not counted as a run. Walk is never switched off.

- `tickets` is read by the widget backend too: keep it on its own line, value only, trailing `/`.
- Area names are lowercase words or hyphens; the name is also the area note's filename. A file belongs to the first area whose prefix matches.
- The body below the frontmatter is free notes for humans; skills ignore it.

### Run counts: `.peep/runs`

Flat `skill: n` lines, one per skill (`ducks: 2`), per machine. Created by the first skill that runs. Not committed: if `.gitignore` lacks `.peep/runs`, say so rather than editing it.

Last step of every Peep skill (the why-line rule): read the file, treat a missing skill line as 0, write `n + 1` back by Edit or Write, and if the old `n` was below 3 end the reply with one `Why: <line>` tied to one of three habits (think before code, finish before starting, prove it works). From the fourth run on, no Why line. A run that stopped early (no setup, bad argument) is not counted.

## 2. Ticket: `<tickets>/<PREFIX>-<NN>-<slug>.md`

`NN` is zero-padded to two digits, `slug` is lowercase with hyphens.

```
---
id: AV-26
title: Short imperative title
status: open
depends: [AV-20, AV-21]
---
```

- `status` is one of `open`, `doing`, `blocked`, `deferred`, `done`, `closed`. `done` is Claude's claim; `closed` is the owner's confirmation and the only status that moves the file to `archive` (with `git mv`).
- `depends`: ids that must be `closed` first; `[]` when none.
- A parked ticket (`blocked` or `deferred`) also has:
  - `reason`: one line, why it is parked.
  - `wake`: the wake-up condition, one of `ticket:AV-26` (that ticket closes), `date:2026-12-01` (that date passes), or free text (Ducks lists it for the owner).
- Leaving parked: back to `open`, delete `reason` and `wake`.
- Other keys (e.g. `attended`) are allowed and kept as-is.
- Body: `# <id> — <title>`, then what to build as a checklist and a "Done when" line. Notes added later go under their own `##` heading.

## 3. Area note: `kb/<area>.md` and `kb/INDEX.md`

One living page per area in the repo setup. Closing a ticket edits the note so it stays true; never append a log.

```
# <area>

## How it works now
## Watch out
## Why it's like this

last touched: AV-26, 2026-11-15
```

- Each section: a few short bullets, present tense. Delete what is no longer true.
- `last touched`: the last ticket that edited the note and the date.
- `kb/INDEX.md`: heading `# Areas`, then one line per area: `- [<area>](<area>.md): <one-line summary>`, in setup order.

## 4. Read-back (Walk, Question Time, Ducks, Reel, Unpack)

What Claude now understands, given after the questions and before anything is written.

- At most 15 lines, grouped under 2 to 4 bold one-word group labels, terse bullets.
- Last line exactly: `Reply "yes", or name what to change.`
- Only "yes" (or plain agreement) lets the skill write. A correction gets a fresh, complete read-back, never a patch, and waits again.

## 5. Reel: `.peep/reel.md`

Reel writes it; the owner may hand-edit it. Question Time and Ducks read it only to flag drift.

```
---
north_star: One line: who it is for, what it does, why this
hook: The one thing that beats the alternatives
niche: The corner of the market it owns
for: Who uses it
non_goals: [no thing one, no thing two]
---
```

- All keys are single lines; `non_goals` is a `[a, b]` list or comma-separated.
- Body: comparables when researched (`- <name> (<link>): how it differs`), else empty.
- Drift: a planned ticket (Question Time) or loose ticket (Ducks) that does not serve `north_star` or hits a `non_goals` entry gets one line naming the ticket and why. Never a block, never a question. No file: no drift line.
