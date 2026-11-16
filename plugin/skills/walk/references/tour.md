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
/peep:goose    use when you want to see how the code fits together, where it is hot, or what touches something
/peep:coderot  use between tickets, to check the tests can actually fail
/peep:bigpeep  use when several tickets are ready and you want them worked in order, one commit each
/peep:tweet    use when stuck on something another session may know, or about to change a file another session is in
```

Shoot sits in Core only when the setup has a `url`; without one, drop line 3, renumber, drop `prove, ` from the heading, and list Shoot first in the Extras block with its use-when line (`/peep:shoot    use after a web UI change, before calling it done`), since it cannot run there. No other skill is staged or hidden.

Next step: `/peep:qt <idea>` when no tickets are open; else `/peep:ducks`.
