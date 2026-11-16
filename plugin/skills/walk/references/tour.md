# Walk: the tour

Printed after the setup is written: the Core block, then the Extras block, in this order. One line per skill: command, then when to reach for it. Drop a line only when `skills.off` names that skill; every other line prints.

```
Core: the everyday rhythm. What, build, prove, carry on, check in.
1 what      /peep:qt       use when an idea or problem is still fuzzy; ends in tickets
2 build     /peep:ticket   use when a ticket is ready to work, close or park
3 prove     /peep:shoot    use after a web UI change, before calling it done
4 carry on  /peep:packup   use when stopping, or context is filling up
5 check in  /peep:ducks    use when sitting down to work, after a Pack Up, or unsure what to pick up next

Extras: reach for these when the moment comes.
/peep:sneak    use when you want to compare 2 or 3 UI directions before building
/peep:goose    use when you need to know what calls what before changing it
/peep:coderot  use between tickets, to check the tests can actually fail
/peep:bigpeep  use when several tickets are ready and you want them worked in order, one commit each
/peep:tweet    use when stuck on something another session may know, or about to change a file another session is in
```

Shoot sits in Core only when the setup has a `url`; without one, drop line 3, renumber, drop `prove, ` from the heading, and list Shoot as a locked Extra.

## Unlocks

An Extra whose unlock is not met prints after the unlocked Extras, under `Locked: they unlock as you go. Each still runs if you call it.`, as its command, two spaces, and its hint, with no use-when line: `/peep:bigpeep  unlocks after 5 tickets`. Every skill stays callable; only the tour and the widget's floppy tray stage them. The widget applies the same rules (`src/unlocks.ts`).

| Skill | Unlocked when | Hint |
|---|---|---|
| bigpeep | `ticket` in `.peep/runs` is 5 or more | `unlocks after 5 tickets` |
| coderot | the setup has `test` and 1 or more tickets are closed | `unlocks after your first ticket closed with a test command in setup` |
| goose | 3 or more tickets are closed | `unlocks after 3 tickets closed` |
| sneak | a closed ticket's file names a path under a UI area | `unlocks after your first UI ticket closed` |
| shoot | the setup has `url` | `unlocks when the setup has a url` |

Closed tickets are the files in the setup's `archive` folder plus any `status: closed` left in `tickets`. A UI area is an `area.<name>` setup key whose name, split on `-`, `_` and `.`, has a part `frontend`, `front`, `ui`, `web`, `client`, `app`, `site` or `www`; the ticket names its path when the path appears at the start of a word (so `src/` does not match inside `src-tauri/src/`). Tweet has no rule and is never locked.

Next step: `/peep:qt <idea>` when no tickets are open; else `/peep:ducks`.
