---
id: AV-93
title: Claude Design prompt for phase 3 art
status: closed
depends: []
attended: true
---

# AV-93 — Claude Design prompt for phase 3 (owner)

**What to deliver:** The owner runs the prompt below in Claude Design and adds the result to `design/skill-pack-2/`.

- [x] Run the prompt; deliverables added to `design/skill-pack-2/`
- [x] Owner says they are final

**Done when:** the phase 3 floppies, tweet emote and graph page style are in `design/skill-pack-2/` and the owner says they're final.

## Prompt

Attach: `design/skill-pack/` (FLOPPY-SPEC.md, tokens.css, emotes.json, sprites/), `design/skill-pack-2/` (phase 1 floppies; phase 2 too once AV-88 is delivered), `design/shell/` (SHELL-SPEC.md, shell.css, demo.html), `design/peep.js`.

> Extend Peep This's floppy and emote sets, matching the existing specs exactly. Floppies: one 16x16 sprite at 16/32/48/96, normal and disabled, dark and light. Emotes: same grid, frame count and stepped timing as `emotes.json`; reduced motion shows frame 1. Tokens only from `tokens.css`, listing new ones. Never redraw the peep.
>
> 1. **Three floppies:** Big Peep (`/peep:bigpeep`, runs a batch of tickets with one helper per ticket and checks on stalled helpers), Tweet Tweet (`/peep:tweet`, a session tells the others it is stuck or sees a conflict), Duck Duck Goose (`/peep:goose`, maps the codebase's dependencies). Label colours distinct from every floppy in the attached sets, in both themes; hover copy in the existing voice.
> 2. **Tweet emote:** shown in a peep's bubble when its session has posted or received a tweet. It must read differently from the clash emote (startled) and must yield to attention states.
> 3. **Graph page style:** a standalone HTML page in the Pixel desktop shell style showing a codebase as area clusters with dependency lines, and live peeps (48px) standing on the areas their sessions are editing. Show a small repo (5 areas) and a big one (20), hover on a node, and a selected area panel. Dark and light.

## Owed before done

- Prompt checked 2026-11-16: attach list names the delivered phase 1 set; the colour rule covers whatever floppy sets are attached. Ready to paste; runs best after AV-88 so phase 2 colours are known.
- Run 2026-11-16 as one merged pass with AV-88 (five floppies, one colour rule over all 14). Delivered in `design/skill-pack-2/`: SPEC.md §07 to §09, tokens.css, sprites/, emotes.json, sprites.py, graph.html, ref/. Canvas: https://claude.ai/artifact/J3QvHUUUg4MejSVxFCxUaN. 
- Decisions for the owner to overrule (SPEC §07 to §09): clash takes the emote slot over tweet, tweet outranks sweat/yawn/zzz; `--em-tweet` aliases `--em-dream`; sheet is 18 columns with 0 to 9 unchanged; graph router is demo only.
- Owner said final 2026-11-16.
