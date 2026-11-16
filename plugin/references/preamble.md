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

## Steps and the next line (F§8)

- At the start of each step named in the skill's frontmatter `steps:`, print one line of its own, exactly `peep step: <name>`, before that step's first tool call. Never save them up and print them together at the end: the widget reads each as it lands. A skipped step prints nothing. The preamble's own work belongs to the skill's first step.
- The fixed output's last line is exactly `next: <command>` or `next: clear to proceed`, and it is the last line of the reply. A line that stops a run early (switched off, nothing written by self-setup or after a read-back, bad argument) and a reply that waits for the owner (a read-back, an offer) carry none.
- A `peep step:` line is not part of the fixed output; "nothing before the header" rules do not count it.

## Last step: run count

Bump `<name>` in `.peep/runs` (F§1): read it, a missing line is 0, write `n + 1` back by Edit or Write. If the old `n` was below 3, put the skill's `Why:` line just before the `next:` line; from the fourth run on, no Why line. A run that stopped early (switched off, nothing written by self-setup, bad argument) is not counted.
