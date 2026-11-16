---
id: AV-126
title: "Design brief: floppy motion, Make it yours page, Goose report page"
status: open
depends: []
---

# AV-126 — Design brief: floppy motion, Make it yours page, Goose report page

**What to build:** Nothing in the app. The owner hands the prompt below to a design session; this ticket is done when its three folders land in `design/` (Question Time 2026-11-16).

- [ ] `design/skill-pack-3/` delivered
- [ ] `design/make-it-yours/` delivered
- [ ] `design/goose-report/` delivered

**Done when:** all three folders hold their `SPEC.md`, `tokens.css` and `ref/` artboards, and the owner accepts the look.

## Prompt

```
You are designing three pieces for Peep This, an always-on-top Windows widget that lists
running Claude Code sessions. Each session row has a "peep" (a pixel marshmallow creature)
on a shelf plank. Skills from the companion "peep" plugin show as 16px floppy disks.
This is design work only: write files under design/, change no app code (src/, src-tauri/, plugin/).

READ FIRST, IN FULL
- design/shell/SHELL-SPEC.md, design/shell/shell.css, design/shell/demo.html
- design/skill-pack/FLOPPY-SPEC.md, design/skill-pack/tokens.css, design/skill-pack/ref/*.png
- design/skill-pack-2/SPEC.md, design/skill-pack-2/tokens.css, design/skill-pack-2/graph.html
- design/skill-pack-2/sprites/ (floppy-<id>-<theme>.png)

HOUSE RULES (apply to all three pieces)
- "Pixel desktop" look: square cards, 2px border, 6px hard black shadow, striped title bars,
  no blur, no glow, no rounded corners. Extend the existing look; do not restyle it.
- Use the real <peep-bot> element from design/peep.js. Never redraw, restyle or add poses.
- Peep sizes 48 / 96 / 40 / 32 only. Floppy is one 16px sprite at 16 / 32 / 48 / 96 only.
- Fonts: Fredoka (display), IBM Plex Sans (UI), IBM Plex Mono (names). Bundled, no network.
- Motion is stepped: 4 to 6 frames over about 160ms, CSS steps(). Reduced motion shows the
  end state static.
- Dark and light themes for everything.
- Do not change: design/peep.js, existing sprites, existing tokens, the existing eject frames.
  New tokens go in new files as additions only.
- Each deliverable folder gets: SPEC.md (numbered rules with exact sizes, offsets, timings,
  like the existing specs), tokens.css (additions only), ref/ artboards for every state
  listed, both themes.

PIECE 1: Floppy insert, drive light and step progress -> design/skill-pack-3/
While a skill runs, its floppy sits seated on that session's plank (right end, 2px in,
bottom 5 of 16 rows clipped; see FLOPPY-SPEC §01). Eject already exists. Design the rest:
- Insert: disk leaves its tray cell and seats on the plank. Mirror of eject.
- Drive light: off, on, blinking. Lit while the skill runs; blinks only while the session
  state is "working".
- Step progress: a skill has 2 to 6 named steps; each is done, current or to do. Must read
  on a 16px floppy on a 48px row and on a 32px floppy on the 96px open-row stage. The step
  name (max 10 characters, IBM Plex Mono) shows on the open row only.
- Finished: last step done, held long enough to read, then the existing eject.
Deliver any new sprites as PNG on 16px cells.

PIECE 2: "Make it yours" settings page -> design/make-it-yours/
One scrolling page inside the widget panel (panel width as the shell's), replacing scattered
settings. Three sections in this order, each under a striped header:
- Peep look: live 96px <peep-bot> preview on a plank; colour (5 palette colours plus
  "from project"); accessory picker. Draw: default (hashed from project), overridden, and
  an accessory not yet earned with its hint.
- Alerts: toast on/off; sound on waiting or error; theme light/dark. Draw each switch on,
  off and focused.
- Pack for this repo: names the repo; each skill as its 32px floppy with an on/off switch;
  question round size (2 to 4); screenshot widths. Draw: a repo with setup, a repo with no
  setup yet (one line and a button), a skill switched off (disabled floppy).
Saving is instant with no Save button; design how a saved change is acknowledged.
Keyboard focus state for every control. Also deliver demo.html, standalone, both themes,
like design/shell/demo.html.

PIECE 3: Goose report page -> design/goose-report/
A standalone browser page (not the widget panel) that a skill generates. One template,
three reports. Start from design/skill-pack-2/graph.html. Layout is yours to propose.
- Code map: areas as cards, import edges between them, live sessions' 32px peeps standing
  on the areas they are editing. Draw: 3 areas, 12 areas, no sessions, two sessions on
  one area.
- Repo health: hot spots (most-changed files), biggest files, untested areas, stale
  tickets, and pattern cards (a lesson, an "explain like I'm 5" fold, a source link).
  Draw: all clear, several findings.
- Custom topic: the owner's question as the title, a written answer, the files involved,
  a small slice of the map. Draw: short answer, long answer.
Shared frame: title bar with report name, repo, date and commit, and a way to move between
the three reports. Must read at 375 and 1280 wide. Theme follows the system setting.
Each report is one self-contained file: inline CSS and JS, fonts and art inlined, no
network. Deliver map.html, health.html, topic.html with dummy data that looks real.

WHEN DONE
List the files written per folder and any rule you could not meet and why. If a decision
is open, pick one, record it in that folder's SPEC.md under "Decisions", and carry on.
```
