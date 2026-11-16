---
name: qt
description: "Question Time: batched rounds of questions on a fuzzy idea, a read-back, then tickets. Use when the user runs /peep:qt."
disable-model-invocation: true
label: Question Time with Peep
steps: [setup, look, rounds, glossary, read-back, draft]
---

# Question Time

The idea is the text after `/peep:qt`. If there is none, ask for it in one line and stop.
"formats" below is the pack's shared file `${CLAUDE_PLUGIN_ROOT}/references/formats.md`. Each heading names its step (`peep step:`, preamble).

## 1. Read the repo setup (step `setup`)

Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count). From the setup take `prefix`, `tickets` and `qt.round` (defaults per formats §1).

## 2. Look before asking (step `look`)

Before the first round, read what the idea touches: `CONTEXT.md` if present, the open and parked tickets in `tickets`, and the code the idea names.
Facts are yours to find, never the user's. Decisions are the user's.
When the idea or an answer contradicts the code, a ticket or `CONTEXT.md`, say so in one line, citing the file or ticket id, and put the contradiction to the user as a question.

## 3. Rounds (step `rounds`)

Work the idea as a tree of decisions. Each round asks the frontier: decisions whose prerequisites are settled.
Always AskUserQuestion rounds of `qt.round` questions, default 4 (fewer only if the frontier is smaller), whatever the run count. No AskUserQuestion tool: say so, ask the frontier once as a numbered list. Shape and wording rules: `references/rounds.md`.

- Recommended option first, its label ending ` (recommended)`.
- Each question stands on its own: no "as above", no answer that depends on another question in the same round.
- After each round, tug loose ends first: a partial answer, an "Other" answer, or an answer that opens a contradiction gets a follow-up round before new ground.
- Every round ends with a "Keep going" question (options "More questions (recommended)" / "That's enough") while the frontier is not empty. When it is empty, say so in one line and go to the read-back.

## 4. Glossary as it lands (step `glossary`)

When an answer settles what a domain term means, add or update it in `CONTEXT.md` straight away, in that file's existing entry style. Glossary entries only: no decisions, plans or ticket notes. No `CONTEXT.md` in the repo: skip this step.

## 5. Read-back (step `read-back`)

Give the read-back exactly per formats §4. Use the groups **Goal**, **Scope**, **Tickets**, adding **Open** only for what the user chose to leave undecided.
The **Tickets** group lists each planned ticket as `<id>: <title>` in dependency order.

If `.peep/reel.md` exists (formats §5) and a planned ticket does not serve its `north_star` or hits a `non_goals` entry, add one line under **Scope**: `drift: <id> <why>`. It never blocks and is not a question.

- "yes" or plain agreement: go to step 6.
- A correction: a fresh, complete read-back, then wait again.
- "stop", "no tickets" or similar: reply `peep: read-back kept, nothing written.` and stop.

## 6. Draft tickets (step `draft`)

Write the tickets from the confirmed read-back, and only those. Format: formats §2. Numbering and file rules: `references/drafting.md`.
Write nothing else: no code, no setup edits.

## Output

After writing, reply with only:

```
peep: <n> tickets written
- <id>: <title>
next: /peep:ticket <id>
```

`next` names the first ticket in dependency order. No recap of the questions or answers.

## Last step: run count

Per the preamble: `Why: ten minutes of questions cost less than rewriting the wrong code.`
