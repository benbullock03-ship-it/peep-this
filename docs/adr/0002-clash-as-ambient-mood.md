# 0002 — Clash as an ambient mood

Date: 2026-11-15 (domain-modelling session). Status: accepted. Overturns, in part, "not taken: collision radar" from the 2026-11-14 mission-control grill. Terms as in `CONTEXT.md`.

## Context

The 2026-11-14 grill rejected a collision radar: watching sessions for work on the same file or branch and warning about it. A warning system needs to be right, and false alarms train users to ignore the widget. With moods (ADR 0001) there is a cheaper way to surface the same fact: peeps can react to each other without the widget claiming anything is wrong.

## Decision

- A **clash** is two different sessions editing the same file (same absolute path) within a few minutes of each other (AV-71: 5 minutes).
- It is a **mood** only: both peeps show the startled emote at each other and the bubble names the file, for a short hold.
- **No warning, no blocking, no trail entry.** No toast, no sound, no state change, no change to sort order, no attention fill. An attention state takes the bubble first, as for every mood.
- Branch overlap and every other part of the radar stay not taken.

## Consequences

- `design/skill-pack/FLOPPY-SPEC.md` ruling 2 calls clash an attention state that pins both rows to the top. This ADR overrules that part: the visual link (dotted bracket when the rows happen to be adjacent, matching bolt tag on both bubbles) is kept; pinning, sorting and the attention fill are not.
- Clash needs the absolute file path in edit hook payloads (AV-54 verifies it) and is derived in the backend mood engine (AV-69), not the session state function.
- A missed or false clash costs nothing but a brief emote, so the detector can stay simple.
