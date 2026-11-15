---
id: AV-87
title: Scrubadubdub skill - two passes, report, fix on yes
status: open
depends: [AV-74, AV-86]
---

# AV-87 — Scrubadubdub skill

**What to build:** `/peep:scrub` scrubs a UI of AI hallmarks and messy spacing and text, and teaches where each comes from.

- [ ] Scope: changes since the last commit on main by default; `--full` for the whole repo
- [ ] Pass 1, grep: the catalogue's grep rules (AV-86); when Impeccable is installed (`.impeccable/` or `npx impeccable`), run `impeccable detect --json` and merge its results into catalogue IDs
- [ ] Pass 2, eye: judge Shoot screenshots at the setup's themes and widths; no `url` in setup: eye findings marked "inferred"
- [ ] Honour `.peep/scrub.md` earned, debt and inline ignores, and DESIGN.md
- [ ] Numbered report: file:line, ID, severity, where it comes from, 1 or 2 alternatives; findings appearing together score above single hits
- [ ] Fix only the numbers the owner approves; every removal names what replaces it; re-scan and report the change
- [ ] Offers `/peep:sneak` when an alternative needs a design choice
- [ ] Why line per AV-74
- [ ] Add the skill to Walk's Extras block with a use-when line

**Done when:** on a test page seeded with five hallmarks, `/peep:scrub` reports all five with alternatives and fixes only the approved ones.
