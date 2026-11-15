# Drafting tickets

- Ids: the next free number after the highest `<prefix>-NN` in `tickets` and `archive` together; never reuse a number.
- Number in dependency order, so a ticket's `depends` only names lower ids or existing tickets.
- File: `<tickets><PREFIX>-<NN>-<slug>.md`, slug from the title, lowercase, hyphens, ASCII only.
- Frontmatter per formats §2: `status: open`, `depends` from the read-back (`[]` when none). Nothing parked unless the read-back says so, then with `reason` and `wake`.
- Body per formats §2: a checklist of what to build from the settled decisions, and a `**Done when:**` line the owner can check. Terms as in `CONTEXT.md`.
- One ticket per read-back line. A ticket that would not fit one session is two tickets; that change needs a fresh read-back first.
- After writing, if setup has `index`, run it once and check it exited 0.
