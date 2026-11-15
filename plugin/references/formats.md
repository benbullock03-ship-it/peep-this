# Pack formats

The four formats every Peep skill reads or writes. Cite the section; do not restate it. Examples are in `examples/`.

All frontmatter is flat: one `key: value` per line, no nesting, no multi-line values. Lists are `[a, b]` or comma-separated. A missing key takes its default.

## 1. Repo setup: `.peep/setup.md`

At the repo root. Ducks writes it on first run; the owner may hand-edit it. Every Peep skill reads it instead of guessing. The `.peep/` folder also holds `handoff.md`, Pack Up's note (AV-60).

| Key | Meaning | Default |
|---|---|---|
| `prefix` | Ticket id prefix, e.g. `AV` | none; Ducks asks |
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

## 4. Read-back (Question Time, Ducks)

What Claude now understands, given after the questions and before anything is written.

- At most 15 lines, grouped under 2 to 4 bold one-word group labels, terse bullets.
- Last line exactly: `Reply "yes", or name what to change.`
- Only "yes" (or plain agreement) lets the skill write. A correction gets a fresh, complete read-back, never a patch, and waits again.
