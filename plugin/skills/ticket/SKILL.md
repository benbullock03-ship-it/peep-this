---
name: ticket
description: "Ticket: work one ticket end to end (build, check, commit only its hunks, mark done), or close or park one. Use when the user runs /peep:ticket <id>, /peep:ticket close <id> or /peep:ticket park <id>, or says 'work AV-12'."
argument-hint: "<id> | close <id> | park <id> [blocked|deferred] <reason> wake <condition>"
label: Ticket
---

# /peep:ticket

One ticket per run. "F§1" to "F§7" below cite the pack's shared file `${CLAUDE_PLUGIN_ROOT}/references/formats.md`. Files in `references/` beside this file are read only when their sub-command runs.

## 0. Repo setup

1. Repo root: `git rev-parse --show-toplevel`. Read `<root>/.peep/setup.md` (F§1), applying its defaults.
2. No file, or no `prefix`: print exactly `peep ticket: no repo setup here. Run /peep:walk first.` and stop. Never guess commands, folders or a prefix. `skills.off` lists `ticket`: F§1 applies.
3. A bare number is an id: `57` means `<prefix>-57`. Ticket file: `<tickets><id>-*.md` (F§2). Not there: look in `<archive>` for that one id only. Never list or read the rest of the archive.

## 1. Sub-command

| Arguments | Do | Detail |
|---|---|---|
| `<id>` | work it: `open` to `doing` to `done` | `references/work.md` |
| `close <id>` | `done` to `closed`, archive, area note, index | `references/close.md` |
| `park <id> ...` | to `blocked` or `deferred` with `reason` and `wake` | `references/park.md` |
| none or other | print the argument hint above and stop | |

Read the detail file before acting, then follow it.

## 2. Rules for every sub-command

- File changes by Edit or Write only. Frontmatter stays flat; keys not yours (e.g. `attended`) are kept.
- Never set `closed` without the owner's yes (see `close.md`). Never park without a reason and a wake-up condition.
- The area note: at most one, `kb/<area>.md`, the first `area.<name>` whose prefix matches the paths in play (F§1). Read it before building; rewrite it only on close (F§3). A tweet fold on close may edit a second one.
- After any status change, run the setup's `index` command if it has one.
- Stage files by name, never `git add -A` or `.`. Leave other sessions' uncommitted files alone. Commit as `<id>: <summary>`. Never push.
- Decisions without a ruling: take the ticket's recommendation or the obvious default and add one line under `## Decisions taken without a ruling` in the ticket. Ask only about something that cannot be undone, in one batch, each with a recommended answer.

## 3. Report

The final reply is these four lines and nothing else: no recap before or after (detail belongs in the ticket).

```
peep ticket: <id> <old status> -> <new status>
commit: <short hash, or none>
checks: build <ok|fail|none>, test <ok|fail|none>[; failures: <new/known/other session, named>]
owed: <what is owed, decided without a ruling, or blocking> | none
```

Session ending or context filling up: suggest `/peep:packup`.

## 4. Last step: run count

Bump `ticket` in `.peep/runs` (F§1). If it was below 3, add a fifth line: `Why: one ticket finished beats three half-built.`
