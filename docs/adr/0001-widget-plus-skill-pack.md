# 0001 — Peep This is the widget plus a skill pack

Date: 2026-11-15 (domain-modelling session). Status: accepted. Terms as in `CONTEXT.md`.

## Context

Until now Peep This was only the widget: a window that shows every live session as a peep, alerts when one needs input, and (from the 2026-11-14 mission-control grill) answers permission requests. It tells you what sessions are doing but not how to work with them. New Claude Code users lack a working rhythm: when to grill an idea, how to work and close tickets, when to Pack Up and `/clear`, whether the tests can fail at all. The owner already runs that rhythm through personal global skills (ticket, handoff, shoot, grills).

## Decision

- **Peep This is the widget and the skill pack, installed as one.** The skill pack is the set of Peep skills delivered as one Claude Code plugin, `peep`, namespaced `/peep:<name>`: `ticket`, `ducks`, `qt` (Question Time), `packup` (Pack Up), `shoot`, `coderot`. Its source lives in this repo under `plugin/` and ships with the widget release; the widget's integration install adds and removes it.
- **Audience: new Claude Code users.** Skills are short, ask batched questions with recommended answers, and give a read-back before writing anything.
- **The widget shows and Jumps.** A Jump copies the command to the clipboard and focuses that session's terminal; the user pastes it. The widget never types into a session and never starts one. Permission decisions (spec §11.1) remain the only answers it gives.
- **Work is tickets with six statuses** (`open`, `doing`, `blocked`, `deferred`, `done`, `closed`). `done` is Claude's claim, `closed` the owner's confirmation and the only status that archives. Parked tickets carry a reason and a wake-up condition. Area notes hold what the code is like now.
- **The widget reflects the pack:** a floppy per Peep skill, an inserted floppy on a row while its skill runs, a floppy tray and a ticket shelf on the open row, moods as emotes in the bubble.
- **Build order:** pack (AV-55 to AV-62), widget (AV-64 to AV-68), moods (AV-69 to AV-71). AV-54 verifies the hook payloads before the widget phase.

## Consequences

- The repo now holds two products in one release: the Tauri app and a Claude Code plugin with its own formats (repo setup, ticket file, area note).
- The backend parses `/peep:` prompts and reads each repo's tickets folder; both stay behind the `AgentSource` seam and in pure state inputs.
- Jump keeps the "never types into a session" line clean at the cost of one paste per action. Writing to a terminal's input stays out.
- The owner's global skills stay; the pack versions are generic and read the repo setup instead of guessing.
