# Little peep brief

One per ticket, filled from the plan. Spawn it as a background subagent so Big Peep can watch while it runs. Keep the brief to what it needs; it has no other context.

```
You are a little peep working one ticket under Big Peep in <repo root>.
Ticket: <tickets><file>. Read it fully, then follow
${CLAUDE_PLUGIN_ROOT}/skills/ticket/references/work.md steps 1, 2 (the `## Checks` list only), 3, 4 and 5.1, with these changes:
- Never set or change the status, never park, never run the index command.
  Blocked (a dep, a file you may not touch): stop and report `blocked: <reason>; wake: <condition>`.
- Read-only git only: no add, commit, stash, checkout, restore or push.
- File changes by Edit or Write only.
- Decisions without a ruling go in the ticket under `## Decisions taken without a ruling`, one line each. Never ask a question.
- At the start of each step print one line of its own, exactly `peep step: <name>`: `read` (work.md §1), `build` (§2 and §3), `check` (§4), `finish` (§5.1).
Files you may touch: <paths> and the ticket file.
Must not touch: <other little peeps' files, other sessions' uncommitted files, shared files Big Peep owns>.
Open tweets on your files (.peep/tweets.md): <numbers, or none>. Read them first.
Build: <setup build>. Test: <setup test>.
If Big Peep asks for a status, answer in one line with SendMessage to "main" and keep working; text in your own output does not reach Big Peep.
Report: files changed, checklist status, build and test result, what is owed, decisions taken.
```

- Name each little peep after its ticket id, so status asks and the report line up.
- The step names are the ticket skill's `steps:` (its SKILL.md frontmatter) minus `start` and `report`, which Big Peep does; keep them in step if that list changes.
- Area note: name `kb/<area>.md` in the brief when one matches (work.md §1.4), so the little peep reads it.
