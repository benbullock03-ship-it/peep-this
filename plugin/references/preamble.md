# Preamble: every Peep skill

Each `SKILL.md` cites this once at the top and keeps only its own Why line. "F§1" and "F§4" cite `formats.md` beside this file.

## Start

1. Repo root: `git rev-parse --show-toplevel`. Read `<root>/.peep/setup.md` (F§1), applying its defaults.
2. `skills.off` names this skill: reply `peep <name>: switched off in .peep/setup.md.` and stop. Walk is never switched off.
3. No setup, or no `prefix`: self-setup below, unless the skill says it runs without one. Then carry on with what was asked.

## Self-setup

1. Guess from the repo as `${CLAUDE_PLUGIN_ROOT}/skills/walk/references/setup.md` step 1. Facts are found, never asked.
2. Print this read-back, at most 5 lines, and wait:

   ```
   peep: no repo setup yet. My guess:
   tickets: <prefix> in <tickets>, index <command | none>
   commands: build <command | none>; test <command | none>; url <url | none>
   areas: <names | none>
   Reply "yes" to write .peep/setup.md and carry on, or name what to change.
   ```

3. "yes": write `.peep/setup.md` per that file's step 5 (body line `Written by /peep:<name> on <YYYY-MM-DD>.`), say `.peep/setup.md new, uncommitted` in one line, then carry on. A correction: a fresh read-back, wait again. "no", "stop" or no reply (headless): reply `peep <name>: nothing written.` and stop.
4. Preference keys stay at their F§1 defaults; `/peep:walk` customises them later.

## Last step: run count

Bump `<name>` in `.peep/runs` (F§1): read it, a missing line is 0, write `n + 1` back by Edit or Write. If the old `n` was below 3, end the reply with the skill's `Why:` line, after its fixed output; from the fourth run on, no Why line. A run that stopped early (switched off, nothing written by self-setup, bad argument) is not counted.
