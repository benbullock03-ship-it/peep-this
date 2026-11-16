---
name: sneak
description: "Sneak Peep: build 2 or 3 throwaway UI variants side by side and keep the owner's pick as the design reference. Use when the user runs /peep:sneak."
disable-model-invocation: true
label: "Sneak Peep: UI variants"
---

# /peep:sneak

Puts a UI design in front of the owner before it touches the app. The UI is the text after `/peep:sneak`; with none, ask for it in one line and stop.
"formats" below is `${CLAUDE_PLUGIN_ROOT}/references/formats.md`.

## 1. Read the repo setup

Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count).

## 2. Look before asking

Read `CONTEXT.md`, `CLAUDE.md`, any existing `design/` folder (its look is the starting point, never restyled), and the code the UI will live in. Facts are yours to find, never the user's. Decisions are the user's.
If the answers already exist from a Question Time or `/peep:shoot --scrub` run in this session, use them and skip step 3.

## 3. Rounds

Ask with the AskUserQuestion tool, 4 questions per round, rules as Question Time's (`${CLAUDE_PLUGIN_ROOT}/skills/qt/SKILL.md` §3, wording in `${CLAUDE_PLUGIN_ROOT}/skills/qt/references/rounds.md`): recommended option first ending ` (recommended)`, each question standalone, loose ends before new ground, a "Keep going" question while the frontier is not empty.
Pointed design questions only: the goal of the screen, who looks at it and when, the one thing that must stand out, density, and what to avoid. Offer concrete options, not "clean and minimal".

## 3b. Read-back

Exactly per formats §4: groups **Screen**, **Look**, **Variants** (the 2 or 3 directions, one line each, genuinely different). "yes" goes to step 4; a correction gets a fresh, complete read-back; "stop" replies `peep: read-back kept, nothing written.` and stops.

## 4. Build the variants

- `<slug>`: lowercase hyphens, from the screen's name.
- If `.gitignore` lacks `.peep/sneak/`, add the line by Edit or Write. This is the one edit outside `.peep/sneak/`.
- Write each variant as `.peep/sneak/<slug>/<variant>/index.html`, variants named `a`, `b`, `c`. Each is self-contained: inline CSS and JS, no network, no build step, dummy data that looks real.
- Match the repo's existing look where it has one; the variants differ in layout and emphasis, not in whether they follow the house style.
- Touch nothing in the app: no `src/`, no config.

## 5. Serve

Run in the background: `node ${CLAUDE_PLUGIN_ROOT}/skills/sneak/scripts/serve.mjs .peep/sneak/<slug>`. It prints `peep sneak: http://127.0.0.1:<port>/`, a page with the variants side by side; each is also at `/<variant>/`. Print that URL, then ask which to keep with AskUserQuestion: options are the variants, plus "None of these" and "Change something first".

## 6. On the pick

- "Change something first": take the change, rebuild the variants, serve again.
- "None": delete `.peep/sneak/<slug>/` after stopping the server, write nothing else.
- A variant: stop the server, move `.peep/sneak/<slug>/<variant>/` to `design/<slug>/` (`design/` kept if it exists, created if not), then delete `.peep/sneak/<slug>/`. Only `design/<slug>/` remains.

## Output

```
peep sneak: <variant> kept as design/<slug>/
nothing in the app changed; a later ticket builds from the reference
```

## Last step: run count

Per the preamble: `Why: seeing three directions costs minutes, rebuilding the wrong one costs a day.`
