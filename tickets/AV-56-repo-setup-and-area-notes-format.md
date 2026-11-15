---
id: AV-56
title: Repo setup and area note formats for the skill pack
status: open
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
