# Walk: the tour

Printed after the setup is written: the Core block, then the Extras block, in this order. One line per skill: command, then when to reach for it. Drop a line only when `skills.off` names that skill; every other line prints, Reel included.

```
Core: the everyday rhythm. Why, what, build, carry on, check in.
1 why       /peep:reel     use when starting a project, or unsure why it is worth finishing
2 what      /peep:qt       use when an idea or problem is still fuzzy; ends in tickets
3 build     /peep:ticket   use when a ticket is ready to work, close or park
4 carry on  /peep:packup   use when stopping, or context is filling up
4 carry on  /peep:unpack   use when starting a session after a Pack Up
5 check in  /peep:ducks    use when sitting down to work, or unsure what to pick up next

Extras: reach for these when the moment comes.
/peep:sneak    use when you want to compare 2 or 3 UI directions before building
/peep:goose    use when you need to know what calls what before changing it
/peep:shoot    use after a web UI change, before calling it done
/peep:coderot  use between tickets, to check the tests can actually fail
/peep:bigpeep  use when several tickets are ready and you want them worked in order, one commit each
/peep:tweet    use when stuck on something another session may know, or about to change a file another session is in
When you need it: /peep:hello (install check)
```

Shoot needs a `url` in the setup, so say so on its line when none was written. Coderot needs `test`; same.

Next step: `/peep:reel` when `.peep/reel.md` does not exist and Reel is not off; else `/peep:qt <idea>` when no tickets are open; else `/peep:ducks`.
