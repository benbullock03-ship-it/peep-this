# Walk: the rhythm tour

Printed after the setup is written, as one block, in this order. One line per skill: command, then what it is for in plain words. Drop a line only when `skills.off` names that skill; every other line prints, Reel included.

```
Rhythm: why, what, build, prove, carry on, check in.
1 why       /peep:reel     pin down who it is for and why it exists
2 what      /peep:qt       questions about an idea, then tickets
2 what      /peep:sneak    2 or 3 UI variants side by side, pick one
3 build     /peep:goose    map what calls what before changing it
3 build     /peep:ticket   work one ticket, commit it, mark it done
4 prove     /peep:shoot    check a web UI change in a real browser
4 prove     /peep:coderot  break code on purpose to see if tests fail
5 carry on  /peep:packup   write the note that opens the next session
5 carry on  /peep:unpack   open from that note, after a repo check
6 check in  /peep:ducks    what is loose, what to pick up next
When you need it: /peep:hello (install check)
```

Shoot needs a `url` in the setup, so say so on its line when none was written. Coderot needs `test`; same.

Next step: `/peep:reel` when `.peep/reel.md` does not exist and Reel is not off; else `/peep:qt <idea>` when no tickets are open; else `/peep:ducks`.
