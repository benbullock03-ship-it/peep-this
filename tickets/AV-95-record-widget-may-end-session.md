---
id: AV-95
title: Record that the widget may end a session
status: done
depends: []
---

# AV-95 — Record that the widget may end a session

**What to build:** Write down the overturned "never kills sessions" rule before any code ships, so the docs and code agree. The CONTEXT.md Widget entry was updated during Question Time on 2026-11-15.

- [x] `docs/adr/0003-widget-may-end-a-session.md`: context (live sessions with no window, e.g. a Claude desktop scratch session), decision (end on a 1s hold from the open row only, TerminateProcess on the PID), consequences
- [x] CLAUDE.md: the Actions bullet no longer says the widget never kills sessions; add ending a session on a hold
- [x] `docs/spec.md`: same change wherever it lists the actions the widget never takes

**Done when:** grepping CLAUDE.md and `docs/spec.md` for "kill" finds nothing that contradicts ADR 0003.

## Notes

- Word-boundary grep (`kill|kills|killed|killing|terminate|terminates`) over CLAUDE.md and `docs/spec.md` finds only the new CLAUDE.md Actions line. Plain substring "kill" also hits "skill"; none of those are about ending sessions.
- spec §2 Non-goals was the only spec line listing actions the widget never takes. §11.10 and §12.9 (not taken) never mentioned killing, so they are unchanged.

## Decisions taken without a ruling

- ADR 0003 includes the `procStart` refusal and "never over HTTP" from AV-96, so the ADR carries the safety guards, not just the gesture.
- ADR 0003 lists the keyboard inbox with strip, flyout and toast as places ending is never offered; it follows from "open row only".
- ADR 0003 records that `TerminateProcess` is a hard kill (no `SessionEnd` hooks, no flush) as a consequence.
- spec §13 intro still cites only ADRs 0001 and 0002; left as is because ending a session is not part of §13.
