---
id: AV-106
title: Tweet hook - clash and stuck nudges
status: doing
depends: []
---

# AV-106 — Tweet hook: clash and stuck nudges

**What to build:** Tweet fires as good practice without being invoked. A plugin hook gives Claude the nudge deterministically; Claude still writes the tweet.

- [x] Plugin `PreToolUse` hook on `Edit|Write|MultiEdit`: if another live session in this repo (`~/.claude/sessions/*.json`, cwd inside the root) edited the same file in the last 30 min (its transcript tail, as `map.mjs` reads it), return `additionalContext` telling Claude to post a clash tweet per `/peep:tweet` before editing
- [x] Same hook, stuck: the same tool failed twice in a row this session. With a live peer in the repo, suggest a stuck tweet; with none, suggest `/clear` carrying what was learned (Anthropic's two-misses rule)
- [x] One nudge per file per session; never blocks the tool; fails silent
- [x] Tweet `SKILL.md` description names the hook as its usual trigger
- [x] Test with fixture sessions and transcripts: clash, no clash, stale edit, stuck with and without a peer

**Done when:** two live sessions editing one file each get the nudge once and a tweet lands on the board, and the fixture tests pass.

## Decisions taken without a ruling

- Matcher is `Edit|Write|MultiEdit|Bash`: the stuck check needs to fire after failed shell commands too; the clash check runs only for the edit tools.
- Stuck = the last two tool results in this session's transcript both `is_error` for the same tool; nudged once per failure pair.
- Reuses goose's `editedFiles` (imported from `skills/goose/scripts/map.mjs`); nudge memory lives in `<tmpdir>/peep-tweet-nudge/<session>.json`.
- "Live" = has a `~/.claude/sessions/*.json`, same as goose; no PID check.

## Owed before done

- Live run: two Claude Code sessions with the plugin loaded edit one file; each gets the nudge once and a tweet lands on `.peep/tweets.md`. Fixture tests pass (`plugin/hooks/tweet-nudge.test.mjs`, 6 tests); the script exits 0 silently on garbage input.
