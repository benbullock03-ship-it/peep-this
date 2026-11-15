# Big Peep output

The final reply, before the Pack Up lines. Nothing before the header line.

```
peep bigpeep: <n> tickets, <n> done, <n> doing, <n> parked, <n> skipped
| ticket | status | commit | notes |
|---|---|---|---|
| <id> <title> | <old> -> <new> | <short hash, or -> | <owed, blocked reason and wake, status asks; or "none"> |
checks: build <ok|fail|none>, test <ok|fail|none>[; failures: <new/known/other session, named>]
decisions: <id>: <one line>   one per line, from each ticket's `## Decisions taken without a ruling`; or "decisions: none"
not committed: <files and why, e.g. index holds another session's rows> | none
push (not run): git push <remote> <branch>
next: /peep:ticket close <id> for each done ticket, else /peep:ticket <next open id> | none
```

- One row per named ticket, in run order, skipped ones included.
- A parked ticket's commit is its park commit.
- Not saved to a file: the tickets and `git log` hold the record, the Pack Up note carries the next step.
