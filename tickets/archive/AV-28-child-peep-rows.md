---
id: AV-28
title: Child peep rows for subagents
status: done
depends: [AV-27, AV-44]
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

## Done 2026-11-14

Live check by Claude at the owner's request, in `npm run tauri dev`:

- **Real fan-out:** 5 parallel subagents from a live session. Children showed with readable bubbles, type and chip. Finished ones showed their result line plus "8s · 1 tool" and lingered, then folded into "✓ done 4" and "done 5". Clicking it reopens them newest first, and clicking again folds them.
- **Synthetic fan-out** of 5 running agents on placeholder `lc-delta` via `SubagentStart`/`PreToolUse` hooks. Real agents were too short-lived: the harness blocks foreground `sleep`, and the auto-mode classifier refused part of a second batch. Results: "+1 more", "– show fewer" and back.
- **Child approvals:** a `Write` request and a destructive `rm -rf` request with `agent_id` mounted under their own children, sorted first, and pulled the hidden fifth child into view. The parent had no card. Allow by mouse worked. An 80ms tap on the destructive card did nothing, and a 1.25s hold allowed it. Both decisions appear on the parent's trail. Child deny was not clicked separately; it is the same AV-23 card whose deny was proven in AV-26 and AV-24.
- **Snapshots:** 7 peep nodes tagged before 3 snapshots were all still there afterwards, none recreated, and the open row stayed open.

## Decisions taken without a ruling

- No Rust change. The snapshot already carries `agents` (AV-27), and `src/model.ts` now types them. The backend `subagents` count is still sent, but the UI ignores it. The peep badge is gone.
- Layout: children hang under the parent, indented to its peep column off a dotted 2px rail in the shelf colour. Each child is a 32px peep on its own 4px-striped plank, a one-line square bubble (ellipsis), then a meta line: agent type (mono), the description dimmed, and a small outlined state chip. The parent row's markup, open stage, facts, trail and card host are unchanged. Children mount after them, in both closed and open rows.
- Child bubble text (`say.ts` `kidSayFor`) follows the session rules. Working says "Running {tool}…". Waiting asks about that agent's held request. Thinking says the task, or the description the agent was launched with. Finished says the result line, or "All done!". An agent ended by its session closing (`idle` + `endedAt`) says "Stopped when the session closed." and its chip reads `Stopped`.
- Finished rows put duration and tool count on the meta line and hide the description (it stays in the name's tooltip), so the stats fit at 360px. The bubble border takes the finished state's colour.
- Accessory: FNV-1a of the trimmed, lower-cased agent type over the 13 accessories other than `none`. `none` is the empty-state house peep's look. An empty type hashes as "agent". Colour is the parent's.
- Linger: 30s from `endedAt`, then the child folds. A timer in `rows.ts` re-renders at the next fold time, so folding happens without a new snapshot.
- "done N" is a button under the children, not on the parent's closed row, because that row is itself a button. Clicking it toggles the folded children back in, newest first. "This turn's list" means every finished agent the backend still holds (up to 20 per session, AV-27). The snapshot has no turn boundary, so finished agents from earlier turns are listed as well.
- Overflow: more than 4 running shows 4 and a "+N more" button, which expands to all and then reads "– show fewer". Running order: children holding an approval first, then attention states, then oldest started. A child with a pending card is therefore never hidden.
- Approvals: a pending request whose `agentId` matches a shown running child mounts its AV-23 card under that child, and only there. The parent's host gets the rest, including requests from agents not in the list. The child shows `waiting` while it holds a request, even before its own state updates.
- Demo: ingest-01 has two children, one holding the shell-tier card. research-02 has 5 running (4 + "+1 more"), one finished child that folds about 26s after load, and 3 already folded ("done 3").
