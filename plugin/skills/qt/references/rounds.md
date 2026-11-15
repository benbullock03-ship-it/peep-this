# Rounds

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
