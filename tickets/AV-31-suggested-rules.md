---
id: AV-31
title: Suggested auto-allow rules
status: todo
depends: [AV-22, AV-26]
attended: false
---

# AV-31 — Suggested auto-allow rules

**What to build:** Stub. The widget offers a rule after the user approves the same shape of request three times, and applies confirmed rules itself (spec §11.8).

- [ ] Shape: project cwd + tool + normalised target (command with arguments generalised, or path glob). Matching is exact on the shape; no fuzzy matching
- [ ] Offer appears on the card after the third matching manual allow: "Auto-allow `npm test` in this project?" Yes / Not now / Never ask
- [ ] Rules live in `%APPDATA%\PeepThis\rules.json`; the widget never writes Claude Code's own permission settings
- [ ] `destructive` tier is never offered and never matched, whatever the rule says
- [ ] Every auto-allow is written to the trail with its rule id and shown as a tick on the row; one click on that entry revokes the rule
- [ ] Settings view lists rules with use counts and a delete button
- [ ] Tests: shape normalisation, threshold, destructive exclusion, revoke

**Done when:** the owner accepted a suggested rule, saw the next matching request pass with no prompt and a tick, then revoked it from the trail.

Decide: how far to generalise arguments (recommended: keep the command and subcommand, drop trailing file arguments; show the exact pattern in the offer).
