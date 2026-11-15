---
id: AV-56
title: Repo setup and area note formats for the skill pack
status: done
depends: [AV-55]
attended: false
---

# AV-56 — Repo setup and area note formats

**What to build:** The two file formats every pack skill reads, written as one shared reference in `plugin/`.

- [ ] Repo setup file at the repo root (name it, e.g. `.peep/setup.md` or frontmatter in one file): ticket prefix, tickets folder, build command, test command, areas (name to paths). Plain enough to hand-edit
- [ ] Ticket file format: frontmatter `id`, `title`, `status` (`open`, `doing`, `blocked`, `deferred`, `done`, `closed`), `depends`, and for parked tickets `reason` and `wake` (`ticket:AV-26`, `date:2026-12-01`, or free text)
- [ ] Area note format in `kb/<area>.md`: "How it works now", "Watch out", "Why it's like this", "last touched"; plus `kb/INDEX.md` one line per area
- [ ] Read-back format shared by Question Time and Ducks: at most 15 lines, grouped, ends asking for "yes" or named changes; a correction triggers a fresh read-back before anything is written
- [ ] One example of each file under `plugin/references/examples/`

**Done when:** AV-57 to AV-59 can cite these formats without restating them.

## Built

`plugin/references/formats.md` (sections 1 to 4) and `plugin/references/examples/` (`.peep/setup.md`, `tickets/WEB-14-refund-button.md`, `kb/checkout.md`, `kb/INDEX.md`, `read-back.md`); linked from `plugin/README.md`.

## Decisions taken without a ruling

- Repo setup is `.peep/setup.md`, flat frontmatter only, so the Rust backend (AV-64) parses `tickets:` with a line split like `scripts/ticket-index.mjs`.
- Areas are `area.<name>: path, path` keys, not a nested map, to keep frontmatter flat.
- Setup also carries `archive`, `index`, `dev`, `url`, `themes`, `widths`; defaults `tickets/`, `<tickets>archive/`, `light, dark`, `375, 1280`.
- Ticket filename `<PREFIX>-<NN>-<slug>.md`, matching this repo; extra frontmatter keys (e.g. `attended`) are kept.
- Leaving parked deletes `reason` and `wake`.
- Read-back groups use bold labels and end with the exact line `Reply "yes", or name what to change.`
- Examples use a fictional `WEB` repo so they are not mistaken for this repo's tickets.
