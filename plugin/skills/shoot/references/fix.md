# On the reply

- `none`: print `peep scrub: nothing changed` and stop.
- A reply can mix forms, e.g. `1 3 debt 4 wrong 6`.

## Numbers to fix

- Fix only the numbers given, across all of a finding's lines. Do not touch anything else in the file, even another hit you notice.
- Every removal names its replacement, taken from the house style: a token, the body face, a scale step. A fix that only deletes leaves the page flat (catalogue, the earned test). Dropping redundant chrome is a replacement only when something else carries its job; name that thing.
- An approved number whose alternative needs a design choice the repo has not made (a new accent, a type pairing, a layout) is not yours to pick. Leave it unfixed and offer `/peep:sneak <screen>` with its alternatives as the directions. Numbers not approved get no Sneak offer.
- Re-run pass 1 on the same scope. Re-shoot and re-run the eye entries for each fixed number, and every eye entry for the elements a fix touched. A new finding caused by a fix is fixed or reverted before the report.

## `debt <n>`

Add or update that ID's `## Debt` line in `.peep/scrub.md` (formats §6) with today's hit count under its path. Debt lines are written only on this reply.

A debt note with fewer hits than its count: lower the count in the same change. Zero deletes the line.

## `wrong <n>`

Ask one question with AskUserQuestion:

- **Earned here** (recommended when the device carries meaning): print the earned line for the owner to add, as `- <id> · <path> · <one sentence on the meaning it carries>`. Never write it; earned lines are the owner's.
- **Rule is wrong** (a grep hit on something that is not the pattern): a false positive is a bug in `rules.mjs` (catalogue). Print the snippet that tripped it, and say it belongs in `rules.test.mjs` as a good case with the regex narrowed. Change the pack only if the owner asks.

## Output

```
peep scrub: fixed <numbers>; left <numbers>
<n>: <what went> -> <what replaced it>   (one line per fixed number)
[sneak: <numbers> need a design choice; run /peep:sneak <screen>]
[debt: <id> · <path> · <count>]   [earned to add: <line>]
before <n> findings, after <m>[; new: <ids>]
next: /peep:sneak <screen> | clear to proceed
```

`next` is `/peep:sneak <screen>` when the sneak line is there. Nothing is staged or committed.
