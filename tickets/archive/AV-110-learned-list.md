---
id: AV-110
title: Learned list page in the tray
status: closed
depends: [AV-109]
---

# AV-110 — Learned list page in the tray

**What to build:** Every tip a user has seen, and every skill Why line, collected on one page they can revisit: "What Peep taught you".

- [x] Tray menu item opens the page; entries grouped by topic (context, rhythm, testing, working with sessions)
- [x] Each entry: the one-line tip, an "Explain like I'm 5" fold, a source link (Anthropic docs, Chroma, the lost-in-the-middle paper, per the report)
- [x] Seen tips stored in widget settings; unseen ones hidden, with a count of how many remain
- [x] Skill Why lines feed it: a skill run seen by the widget adds that skill's lesson
- [x] Architecture patterns feed it: each `plugin/references/patterns.md` entry (lesson, ELI5, source) once a Ducks report card has cited it (AV-105 ruling, 2026-11-16)
- [x] Pixel-shell look; Shoot both themes

**Done when:** after triggering two tips and running one skill, the page lists three entries with working folds and links; the owner confirms what they saw.

## Decisions taken without a ruling

- The page is a panel view like settings (`#learned`, `src/learned.ts`), not a new window: tray "What Peep taught you" opens the panel and emits `open-learned`. No capability change.
- Settings key `learned` = `{ tips, skills, patterns }` (ids). AV-109's `tipsSeen` is folded in on load and left in place, since AV-109 still reads it. Crowded counts once shown, though it is a once-per-session tip.
- A tip counts when it is on show on a row (`Session.tip`), a skill when a session's `skill` is set (`/peep:<skill>` or the Skill tool). Patterns are read from `<repo>/.peep/graph/candidates.json` card `pattern` ids each time the page opens, for every repo root the widget knows.
- A fifth topic, `architecture`, holds the patterns and Goose: the ticket's four topics have no place for them.
- Every Peep skill with a Why line has an entry (13); its ELI5 and source are new copy in `src/learned.ts`. Patterns copy their lesson, ELI5 and book source from `patterns.md`. `learned.test.ts` fails if a Why line or pattern id drifts from the plugin.
- Sources (checked 2026-11-16, all 200): Chroma context rot, arXiv 2307.03172 (lost in the middle), code.claude.com best practices, sub-agents and costs, platform.claude.com prompt caching, Wikipedia mutation testing. Book sources show as text, no link.
- Links open in the default browser through a new `open_source` command that accepts only `https://` URLs (reuses the tray's `ShellExecuteW` path).
- Browser check: `?demo&learned` opens the page as after two tips, one skill and one cited pattern. Shot dark/light at 375 and dark at 1280: 4 entries in 4 groups, fold opens, `20 more to find as you go.`, no horizontal overflow (`.playwright-mcp/av110-*.png`).

- Live check 2026-11-16 in `tauri dev` (CDP): `open-learned` showed this machine's 4 entries (crowded; 3 patterns from this repo's Ducks report); Chroma link opened Firefox on the Context Rot page. Found and fixed: the window shrank to the header because window fit measured only rows and settings; `#learned` is now measured, and the digest card waits while it is open.

## Owed before done

- Owner close. Not seen by Claude: the native tray menu item itself (the click emits the same `open-learned` used above).
