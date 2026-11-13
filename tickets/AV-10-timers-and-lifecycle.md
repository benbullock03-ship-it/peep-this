---
id: AV-10
title: Stuck timer and ended-session fade
status: todo
depends: [AV-03, AV-04]
attended: false
---

# AV-10 — Stuck timer and ended-session fade

**What to build:** A backend ticker that feeds `Tick` events into the state engine (stuck after 6 min), and the ended-session behaviour from spec §3.7.

- [ ] Tick every 5s; only sessions in `thinking`/`working` are checked
- [ ] Ended session: final state `success` if the last turn ended cleanly else `idle`, row dimmed, removed 60s after `endedAt`
- [ ] Removal fades out in the UI rather than popping

**Done when:** a killed session dims and leaves after a minute, and a test covers the stuck transition through the ticker.
