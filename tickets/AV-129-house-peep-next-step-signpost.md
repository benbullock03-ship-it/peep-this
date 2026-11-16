---
id: AV-129
title: House peep next-step signpost
status: open
depends: [AV-124]
---

# AV-129 — House peep next-step signpost

**What to build:** One signpost that always names the next step, as a Jump (Question Time 2026-11-16). This is the hook: Peep This tells you what to do next.

- [ ] The house peep's bubble (empty state) shows one line and its command: integration not installed: "Install the integration"; installed, no session yet: "Start Claude Code in a project"
- [ ] A session's row, when `idle` or `success` with no attention state, shows the next step in its bubble: no `.peep/setup.md` in its repo: `/peep:walk`; else the `next:` line from the last Peep skill run in that session (AV-124); else nothing
- [ ] Clicking the bubble's command is a Jump: copy it and focus that session's terminal. It never types into the session
- [ ] An attention state or a mood takes the bubble first; the signpost never pulses, toasts or sorts the row
- [ ] Backend: capture the `next:` line from the skill's final output; `cargo test` with a transcript fixture. Vitest for the bubble's precedence

**Done when:** in a repo with no setup an idle row offers `/peep:walk`; after `/peep:ducks` it offers that run's `next:` command and clicking copies it; the owner confirms in `npm run tauri dev`.
