---
id: AV-46
title: Peep animation CPU against the §8 budget
status: todo
depends: []
attended: true
---

# AV-46 — Peep animation CPU against the §8 budget

**What to build:** Find out whether the widget stays under spec §8's "idle: under 1% CPU" and bring it under if not. AV-35 measured a release build with 3 busy sessions at 0.91% and 1.07% with a transparent window, and 1.15% and 1.22% with an opaque one. The cost is the peeps animating, not the window.

- [ ] Owner ruling: does §8 "idle" mean no session busy, or the widget left alone with whatever the sessions are doing?
- [ ] Measure a release build for 2 × 30s with 0, 3 and 10 sessions, both all idle and all busy
- [ ] If over budget: cap peep fps in rows (the open-row stage keeps full fps), or pause peeps while the card is fully covered or minimised to the strip
- [ ] Spec §8 states which case the number applies to

**Done when:** the measured case §8 names is under 1% on a release build.
