---
id: AV-77
title: Walk Through skill - first run, rhythm tour, per-skill preferences
status: closed
depends: [AV-74, AV-75]
---

# AV-77 — Walk Through skill

**What to build:** `/peep:walk` orients a new user and tailors the pack to their repo. It owns first run.

- [ ] Tours the skills in rhythm order: The Reel (why), Question Time (what), Ticket (build), Shoot and Coderot (prove), Pack Up and Unpack (carry on), Ducks (check in); the rest marked "when you need it"
- [ ] Batched rounds about the repo (prefix, commands, areas, themes, widths) and how the owner works
- [ ] Writes `.peep/setup.md` (formats §1) plus flat preference keys: `qt.round`, `packup.questions`, `shoot.widths`, `skills.off`; document them in `formats.md`; each skill reads its keys
- [ ] Ends by suggesting the first real step, usually `/peep:reel`
- [ ] Ducks with no setup now replies that Walk Through sets the repo up, instead of writing setup itself (edit `plugin/skills/ducks`)
- [ ] Why line per AV-74

**Done when:** in a repo with no `.peep/`, `/peep:walk` writes a setup that Ducks and Ticket then read, and `/peep:ducks` there first points to Walk Through.

## Decisions taken without a ruling

- Key meanings: `qt.round` = questions per Question Time round, 2 to 4, default 4; `packup.questions` = `ask` (default) or `never`; `shoot.widths` overrides `widths` for Shoot only; `skills.off` = comma-separated skill names.
- Only keys that differ from their default are written to setup.
- `skills.off` rule lives once in formats §1 (reply `peep <name>: switched off in .peep/setup.md.`, not counted); each of qt, ticket, ducks, coderot, reel, packup, shoot, unpack carries a one-line cite. Walk can never be switched off. Hello is not covered.
- Two rounds in Walk (repo, then how the owner works); round B skips themes/widths when no `url`.
- Old `no repo setup` messages in qt, ticket, reel, coderot and Ducks now say `/peep:walk`. `ducks/references/setup.md` moved to `walk/references/setup.md` and deleted.
- Walk's last-step habit: "a minute of setup now saves every later skill from guessing" (think before code).

## Owed before close

- Live check: `/peep:walk` in a repo with no `.peep/` writes a setup that Ducks and Ticket read; `/peep:ducks` there first points to Walk Through. Not run in the batch.
