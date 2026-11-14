---
id: AV-29
title: Background shell tracking with health states
status: todo
depends: []
attended: false
---

# AV-29 — Background shell tracking with health states

**What to build:** Stub. Track each background shell a session starts and derive one of five states (spec §11.7): `running`, `quiet`, `serving`, `exited`, `failed`.

- [ ] **Verify** how a background shell appears in hook payloads: the `run_in_background` input, the shell id in the tool response, the tools that read output and kill it, and whether any event reports exit and exit code
- [ ] **Verify** whether the shell's OS PID can be found (child of the session PID with a matching command line)
- [ ] `quiet`: no output observed for 2 min (only if output is observable; else drop the state and record why)
- [ ] `serving`: a process in the shell's tree holds a listening TCP port; show the port
- [ ] `failed`: non-zero exit, with exit code and last error line if available
- [ ] `Session.shells: Shell[]` with id, command, state, startedAt, port, exit code, last line
- [ ] Shells end with the session; orphan tracking is out of scope

**Done when:** logs for a live session running a dev server and a failing build show `serving :<port>` for one and `failed` with its exit code for the other.
