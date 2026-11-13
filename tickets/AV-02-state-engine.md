---
id: AV-02
title: Session model and pure state engine
status: todo
depends: [AV-01]
attended: false
---

# AV-02 — Session model and pure state engine

**What to build:** `src-tauri/src/state.rs` with `Session`, `State` (peep's seven), `Event` (hook events, session-file status, transcript facts, `Tick`) and `fn apply(session, event, now) -> Session`, implementing spec §4 exactly. No I/O.

- [ ] Types match spec §5.3 (serde, camelCase) so the snapshot serialises straight to the frontend
- [ ] Every row of the §4 table and every rule below it has a unit test
- [ ] Stuck: 6 min without an event while `thinking`/`working`, via `Tick`
- [ ] `success` held until `UserPromptSubmit`; idle-reminder `Notification` does not override it
- [ ] Subagent count increments/decrements/clamps/resets
- [ ] Task text per state; `working` shows `<Tool> <target>`, target truncated to 60 chars
- [ ] Hook-less fallback: session-file `busy`/`idle` maps to `working`/`idle`

**Done when:** `cargo test state` passes with the cases above.

Decide: whether hook-derived state should ever be overridden by the session file's `status` (recommended: only when no hook event arrived in the last 10s).
