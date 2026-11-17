---
id: AV-145
title: Rhythm first-use dates must not wrap mid-date
status: closed
depends: []
---

# AV-145: Rhythm first-use dates must not wrap mid-date

**What to build:** on the Learned page's "Your rhythm" list (AV-137) the first-use date breaks across lines ("first 2026-" / "11-17") at the 360px panel width, seen in the live check on 2026-11-17.

- [x] The date (or the whole "first YYYY-MM-DD" part) never breaks inside itself; it moves to the next line whole
- [x] Shoot `?demo&learned` at the panel width and confirm

**Done when:** at 360px each rhythm line shows its date unbroken.

## Checks

1. pass: at 360px on `?demo&learned` each `.rhythm-date` has one client rect ("first 2026-11-13", "first 2026-11-11"); screenshot `.playwright-mcp/av145-rhythm.png` shows each date whole on its own line.
2. pass: clipboard summary unchanged (`rhythmSummary` untouched; `learned.test.ts` passes).

## Done 2026-11-17

`src/learned.ts` renders "first <date>" as `span.rhythm-date`, `src/styles.css` `.rhythm-date { white-space: nowrap }`.

## Decisions taken without a ruling

- No new unit test: the change is a render split with no DOM test harness in this repo; the Shoot check covers it.
