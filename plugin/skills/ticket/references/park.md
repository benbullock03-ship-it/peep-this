# /peep:ticket park <id>: set it aside without losing it

## 1. Refuse without both parts

The arguments after the id must give a reason and a wake-up condition. If either is missing, reply with exactly this, nothing before or after, and stop, changing nothing:

```
peep ticket: park needs a reason and a wake-up condition, e.g.
/peep:ticket park AV-12 blocked "no API key yet" wake ticket:AV-09
/peep:ticket park AV-12 deferred "after launch" wake date:2026-12-01
```

A `closed` ticket cannot be parked: say so and stop.

## 2. Normalise

- Status: `blocked` or `deferred` if given. Not given: `blocked` when the wake is a ticket, else `deferred`.
- `reason`: one line, the owner's words, no quotes.
- `wake` (F§2): "when AV-9 closes" or an id becomes `ticket:<PREFIX>-<NN>`; a date becomes `date:YYYY-MM-DD`; anything else stays as free text on one line.

## 3. Write

1. Set `status`, then put `reason:` and `wake:` on the lines right after it. Replace any old `reason`/`wake`. Keep every other key.
2. Run the setup's `index` command, if any.
3. Stage the ticket and the index file by name. Commit `<id>: park (<status>)`.
