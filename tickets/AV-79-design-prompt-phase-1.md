---
id: AV-79
title: Claude Design prompt for phase 1 art
status: open
depends: []
attended: true
---

# AV-79 — Claude Design prompt for phase 1 art (owner)

**What to deliver:** The owner pastes the prompt below into Claude Design with the listed files attached, and brings the result back as `design/skill-pack-2/`, which becomes the reference like `design/skill-pack/`.

- [x] Run the prompt in Claude Design (2026-11-15, canvas https://claude.ai/artifact/FEirV38AYhxsvRYcrBg6gA)
- [ ] Deliverables in `design/skill-pack-2/`: spec, tokens additions, sprites, `ref/` mocks, dark and light. In place except PNG mocks: `ref/` holds the artboard sources, which need the canvas to render; PNGs come from the canvas export.
- [ ] Owner says they are final

**Done when:** the files are in `design/skill-pack-2/` and the owner says they're final.

## Prompt

Attach: `design/shell/` (SHELL-SPEC.md, shell.css, demo.html, ref-C-pixel.png), `design/skill-pack/` (FLOPPY-SPEC.md, tokens.css, emotes.json, ref/), `design/peep.js`, `CONTEXT.md`.

> You are extending Peep This, an always-on-top Windows widget where each Claude Code session is a pixel marshmallow ("peep") on a striped shelf plank. The look is fixed: the "Pixel desktop" shell in `design/shell/` (square card, 2px border, 6px hard black shadow with no blur, striped title bar, square speech bubbles, planks) and the floppy layer in `design/skill-pack/`. Adapt it; do not restyle it. Use only tokens from `tokens.css`, adding new ones where needed and listing them. Peep sizes are fixed at 48 (row), 96 (open-row stage), 40 (strip), 32 (subagent child). Floppies are one 16x16 sprite at 16/32/48/96. Motion is stepped, 4 to 6 frames over about 160ms; reduced motion shows frame 1 static. Never redraw the peep: use `<peep-bot>` stand-ins. Deliver dark and light for everything, as source: a spec markdown with rulings, a tokens.css addition, PNG sprites and ref mocks, like `design/skill-pack/`.
>
> Design these:
>
> 1. **Closed shelf.** A section listing sessions that ended in the last 8 hours, newest first. Each row: the session's peep (dimmed or asleep, your call), its last bubble text, session name in IBM Plex Mono, the ticket it was on, an end-state chip, and up to four flags: uncommitted changes, ticket left doing, no Pack Up note, ended waiting/error. Flagged rows must read as "you may have forgotten this" without alarm. Clicking a row copies `claude --resume <id>`: draw the copied moment, reusing the Jump "copied" pattern. Show empty, 1, 6 and 20 rows, and where the shelf sits in the open window relative to live rows.
> 2. **Rate-limit pressure on the title.** The "Peep This" title in the striped title bar reacts as the 5-hour or 7-day rate limit fills up: calm below 80%, a step at 80%, stronger at 95%, a distinct state at 100%. Keep it pixel and stepped; no glow, no blur.
> 3. **Header limit meters with numbers.** Today the 5h and 7d meters are 10 square blocks with the % only on hover. Add the % beside each meter legibly at the widget's size. Amber from 70%, red from 90%, as the existing bars.
> 4. **Limit bubble.** At 80% every peep shows the existing sweat emote with bubble text like "5h limit 82%". At 95% and 100% a Windows toast fires: draw the toast copy, plus the reset time when known. Show how it yields to an attention state.
> 5. **Arc line.** One line per repo on the open row: the north star (one line of text, truncated), closed / open / parked ticket counts, and the last ticket closed. It should make finishing feel like progress. Show it with and without a north star.
> 6. **Three new floppies**, matching the six in FLOPPY-SPEC: Unpack (`/peep:unpack`, opens a session from a Pack Up note), Walk Through (`/peep:walk`, first-run tour that tailors the pack), The Reel (`/peep:reel`, sets the project's north star). Pick label colours distinct from the existing six in both themes, plus hover card copy in the existing voice.
