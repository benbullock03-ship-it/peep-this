# Rounds

## Two modes

- **Ask** (`qt` run count below 3, or any `qt.round` in the setup): AskUserQuestion rounds per the shape below, `qt.round` questions each, default 4.
- **List** (`qt` run count 3 or more, no `qt.round`): the whole frontier as one numbered text list, see "List shape". The reply is "agree all" or named overrides (`2: SQLite, 5: drop it`); unnamed items take their recommendation. Loose ends and the next frontier come back as a fresh list.

## List shape

```
1. <Header>: <question>? Recommend: <option>, <why in a few words>. Else: <alt>; <alt>.
2. ...
Reply "agree all", or name overrides (e.g. "2: <alt>").
```

- Same rules as the ask shape: one sentence per question, recommendation first with its reason, real alternatives only, each item readable alone.
- No "Keep going" item: the list is the whole frontier. When the next frontier is empty, go to the read-back.

## AskUserQuestion shape

- One call per round, up to 4 questions. The tool allows 2 to 4 options per question and adds "Other" itself; do not add your own "Other".
- `header`: at most 12 characters, a noun naming the decision (`Storage`, `Auth`, `Scope`).
- `question`: one sentence ending in `?`, naming the thing decided, readable without the rest of the round.
- First option is the recommendation; its label ends ` (recommended)`. Its `description` says why in one line, citing the code, ticket or term that supports it when there is one.
- Other options: real alternatives, each with a one-line consequence. No straw men.
- `multiSelect: true` only when choices genuinely combine (e.g. which platforms).

## Choosing the frontier

- A question belongs in this round only if every decision it depends on is settled.
- Order inside a round: biggest consequence first.
- Never ask what a file, ticket or command can answer. Look it up, then ask only the decision it leaves open.
- A contradiction found while looking: one line in the message before the round (`Note: tickets/AV-12 says X, the idea says Y.`), and a question that settles it.

## Loose ends

After answers come back, before new ground:

- "Other" with free text: restate it as a concrete choice and confirm it in the next round, or fold it in if unambiguous.
- Partial answer (picked an option but added a condition): a follow-up question on the condition.
- Answer that conflicts with an earlier one or with the repo: one question that settles which wins.

## Keep going

While the frontier is not empty after this round, the round's last question is:

- header `Keep going`, question `Keep going with more questions?`
- options `More questions (recommended)` / `That's enough`

That question counts toward the 4. "That's enough" goes straight to the read-back; unsettled decisions go under **Open**.

## Between rounds

At most one short line of text before each round (a note, or nothing). No recap of earlier answers.
