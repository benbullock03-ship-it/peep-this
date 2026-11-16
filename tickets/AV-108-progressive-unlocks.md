---
id: AV-108
title: Progressive unlocks in the Walk tour and floppies
status: doing
depends: [AV-103]
---

# AV-108 — Progressive unlocks in the Walk tour and floppies

**What to build:** A beginner sees the core rhythm first; Extras appear as they earn them. Skills stay callable at any time; only the tour and floppies stage them.

- [x] Unlocks from `.peep/runs` and tickets: Big Peep after 5 `ticket` runs; Coderot after the first ticket closed with a test command in setup; Goose after 3 tickets closed; Sneak and Scrub after the first ticket closed that touched a UI area; Shoot in Core when setup has a `url`
- [x] Walk tour prints locked Extras greyed with their unlock, e.g. `/peep:bigpeep  unlocks after 5 tickets`
- [x] Widget floppy tray shows locked floppies dimmed with the unlock on hover; Jump still works
- [x] Big Peep's first-run Why: watch the diffs, not the reports
- [x] Vitest for the unlock rules

**Done when:** a fresh repo's tour shows locked Extras with their unlocks, this repo's tour shows them unlocked, and the floppy tray matches; the owner confirms the tray.

## Decisions taken without a ruling

- Rules live once in `plugin/skills/walk/references/tour.md` (Unlocks table) and are mirrored in `src/unlocks.ts`; no formats.md note, since only Walk and the widget read them.
- "Touched a UI area" = a closed ticket's file names a path under an `area.<name>` whose name has a part frontend, front, ui, web, client, app, site or www; matched at a word start so `src/` does not hit `src-tauri/src/`. Git history is not read.
- Closed = archive files plus `status: closed` still in `tickets/`, same as the arc line's count.
- Shoot is Core line 4 `prove` (after build) when the setup has a `url`; without one it is a locked Extra.
- A terminal cannot grey text, so locked Extras print in their own `Locked:` block after the unlocked ones.
- Backend sends `Repo.unlock` (`ticketRuns`, `closedUi`, `test`, `url`); `arc.closed` supplies the closed count. No repo or no `unlock` (browser demo) locks nothing.
- Locked disks dim the sprite (grayscale, 45% opacity) and get a dashed border; the hover card adds `Locked: <hint>. Jump still copies it.` in full text colour (kb: never dim text by opacity).
- Big Peep's Why replaced the old one: `Why: watch the diffs, not the reports; a report is a claim, the diff is the evidence.`

## Owed before done

- Shoot the dimmed tray in both themes (`?demo` repos carry no `unlock`, so a demo repo needs one in `src/demo.ts`), and the owner confirms the tray in `npm run tauri dev`.
- Live tour check: `/peep:walk` in a fresh repo shows the Locked block; in this repo (19 ticket runs, test, url, closed UI tickets) shows none.
