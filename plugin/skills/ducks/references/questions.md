# Questions

One AskUserQuestion call. Shape per Question Time's rules: `header` at most 12 characters, `question` one sentence ending in `?`, 2 to 4 options, recommendation first ending ` (recommended)`, a one-line description each, no own "Other".

## Which, in this order (keep the first 4)

| Loose end | header | question | options |
|---|---|---|---|
| Stale `doing` | `Stale <id>` | `<id>` has been doing with no change for <n> days. Finish, park with a reason, or close? | "Finish it next (recommended)" / "Park it with a reason" / "Close it". Park: the owner runs `/peep:ticket park <id>` with the reason. Close: a `close <id>` line in **Do** |
| `done` tickets | `Close` | Close these done tickets: `<ids>`? | one done ticket: "Close <id> (recommended)" / "Leave it in done". Two to four: `multiSelect: true`, one option per id, the first labelled `(recommended)`, description says all are recommended. More than four: "Close all <n> (recommended)" / "Leave them all"; name a subset via Other |
| Free-text wake | `Wake <id>` | `<id>` waits on "<wake>". Is that met now? | "Not yet, keep it parked (recommended)" / "Met, wake it" |
| Handoff ruling | from the note | the note's ruling, as a question | the note's recommended answer first |
| Parked, broken | `Fix <id>` | `<id>` is parked without a reason or wake. What now? | "Back to open (recommended)" / "Leave it for /peep:ticket park" |

Several stale tickets, free-text wakes or rulings: one question each, in ticket order, until the 4 run out. A stale ticket is never a reason to refuse new work. Left-over decisions go under **Loose** as `undecided: <what>`, for the next run.

## Not asked

- Wakes whose condition is met: no question, they go straight to **Do**. The owner can strike one in the read-back.
- Failing tests, uncommitted work, rules that bite: facts, listed under **Loose** or **Watch**. Ducks never fixes, stashes or commits them.
