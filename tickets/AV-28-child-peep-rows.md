---
id: AV-28
title: Child peep rows for subagents
status: todo
depends: [AV-27]
attended: false
---

# AV-28 — Child peep rows for subagents

**What to build:** Stub. Indented child rows under the parent, one per running subagent (spec §11.6), replacing the count badge.

- [ ] Each child: 32px peep in the parent's colour, accessory hashed from the agent type (same type, same accessory, in every project), agent name, state LED, current tool
- [ ] `design/peep.js` used unchanged; no new poses or sizes
- [ ] On finish: child shows `success` or `error` with the first line of its result, duration and tool count for 30s, then folds into a "done N" counter on the parent; clicking the counter reopens this turn's list
- [ ] More than 4 running: show 4 and a "+N more" row that expands
- [ ] A child with a pending approval shows AV-23's card under itself
- [ ] Vitest: agent-type accessory hash, fold timing, overflow

**Done when:** the owner watched a real fan-out and could say what each agent was doing without opening the terminal.
