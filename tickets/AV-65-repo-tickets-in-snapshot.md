---
id: AV-65
title: Backend reads each project's open and parked tickets
status: open
depends: [AV-56]
attended: false
---

# AV-65 — Ticket shelf data

**What to build:** For each session's repo, the snapshot carries its open and parked tickets.

- [ ] Find the tickets folder from the repo setup, else `tickets/`; watch it for changes
- [ ] Parse frontmatter only: id, title, status, reason, wake; skip archive
- [ ] Sessions in the same repo share one read
- [ ] Watches the repo's archive to notice a ticket being closed (feeds the cheer mood, AV-70)
- [ ] `cargo test` for the parser with fixtures, including a malformed file (skipped, not fatal)

**Done when:** the snapshot lists a live repo's open and parked tickets and updates when one changes.
