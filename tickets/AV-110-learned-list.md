---
id: AV-110
title: Learned list page in the tray
status: blocked
reason: waiting on AV-109
wake: ticket:AV-109
depends: [AV-109]
---

# AV-110 — Learned list page in the tray

**What to build:** Every tip a user has seen, and every skill Why line, collected on one page they can revisit: "What Peep taught you".

- [ ] Tray menu item opens the page; entries grouped by topic (context, rhythm, testing, working with sessions)
- [ ] Each entry: the one-line tip, an "Explain like I'm 5" fold, a source link (Anthropic docs, Chroma, the lost-in-the-middle paper, per the report)
- [ ] Seen tips stored in widget settings; unseen ones hidden, with a count of how many remain
- [ ] Skill Why lines feed it: a skill run seen by the widget adds that skill's lesson
- [ ] Architecture patterns feed it: each `plugin/references/patterns.md` entry (lesson, ELI5, source) once a Ducks report card has cited it (AV-105 ruling, 2026-11-16)
- [ ] Pixel-shell look; Shoot both themes

**Done when:** after triggering two tips and running one skill, the page lists three entries with working folds and links; the owner confirms what they saw.
