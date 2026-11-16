# Walk: the north star round

Round C, after rounds A and B. It gives the project a reason to finish. Nothing is written before the yes to the whole read-back.

## 1. Look first (do not print)

Read `CONTEXT.md`, the README and `CLAUDE.md` if present, and `.peep/reel.md` if it exists. Guess who it is for, what hurts for them today, the alternatives they have now, and what it will not do. A rerun starts from the file's values and each question names the current one.

## 2. Round C

One AskUserQuestion call, 4 questions, shape per `${CLAUDE_PLUGIN_ROOT}/skills/qt/references/rounds.md`, guess first with ` (recommended)`:

| header | question | options |
|---|---|---|
| `For` | Who uses this project, and in what moment? | the guess; one other plausible user |
| `Problem` | What hurts for them today? | the guess; one other plausible pain |
| `Hook` | What makes them pick this over what they use now? | the guess, naming the alternative it beats; one other; "Look up comparables first" |
| `Non-goals` | What will it never do, even when asked? | the guessed list, comma-separated; "None yet" |

"Look up comparables first": search the web for at most 5 comparable products, keep name, link and one line on how each differs, then ask `Hook` again as one question with the comparables in its descriptions. No search without that pick.

If no answers come back, the read-back carries the guesses marked `(unconfirmed)` and Walk stops.

## 3. Read-back group

One group, **North star**, in Walk's read-back:

- the north star: one line, for whom, what, why this (from For and Problem)
- `hook:` the Hook answer
- `not:` the non-goals
- `comparables: <n>` only when researched

## 4. Write on yes

`.peep/reel.md` per F§5, replacing any earlier file: `north_star`, `hook`, `for`, `non_goals`; `niche` only when an answer named one. Body: the comparables as `- <name> (<link>): how it differs`, else empty. Do not stage or commit it. Say `.peep/reel.md new, uncommitted` (or `updated, uncommitted`).
