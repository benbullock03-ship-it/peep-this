---
id: AV-46
title: Peep animation CPU against the §8 budget
status: doing
depends: []
attended: true
---

# AV-46 — Peep animation CPU against the §8 budget

**What to build:** Find out whether the widget stays under spec §8's "idle: under 1% CPU" and bring it under if not. AV-35 measured a release build with 3 busy sessions at 0.91% and 1.07% with a transparent window, and 1.15% and 1.22% with an opaque one. The cost is the peeps animating, not the window.

- [x] Owner ruling: does §8 "idle" mean no session busy, or the widget left alone with whatever the sessions are doing? **No session busy** (2026-11-15)
- [ ] Measure a release build for 2 × 30s with 0, 3 and 10 sessions, both all idle and all busy
- [ ] If over budget: cap peep fps in rows (the open-row stage keeps full fps), or pause peeps while the card is fully covered or minimised to the strip
- [x] Spec §8 states which case the number applies to

**Done when:** the measured case §8 names is under 1% on a release build.

## Owed before done

- The measurement. The case §8 now names needs every session idle, and the session doing the measuring is itself busy, so it was not taken in this run. Measure `peep-this.exe` for 2 × 30s with 0, 3 and 10 idle sessions from a plain terminal, not from a Claude session.
- AV-35's busy figures (0.91% to 1.22% with 3 busy) are now outside the budget's scope; nothing to cap unless the idle number is over.
