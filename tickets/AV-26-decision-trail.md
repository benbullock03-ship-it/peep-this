---
id: AV-26
title: Per-session decision trail
status: todo
depends: [AV-21]
attended: false
---

# AV-26 — Per-session decision trail

**What to build:** A local record of every approval outcome (spec §11.5), shown on the expanded row and searchable in settings.

- [ ] Append-only JSONL at `%APPDATA%\PeepThis\trail\<yyyy-mm>.jsonl`: time, session id and name, cwd, tool, target (command or path), tier, outcome (`allowed`, `denied`, `released`, `auto-allowed`), deny reason, rule id when a rule fired
- [ ] Stores the command or path, never file contents or diffs. This is the one exception to "no tool content on disk" (spec §6.5, §8); a setting turns the trail off
- [ ] Expanded row shows the session's last 5 as ticks and crosses with tool and target; hover shows time and reason
- [ ] Settings view: full log, filter by session, outcome and text
- [ ] Retention: 90 days, pruned on startup
- [ ] Tests: write, read back, prune, malformed line skipped

**Done when:** the owner saw their own allow, deny and a timed-out release for a live session listed correctly on its expanded row and in the log.
