---
id: AV-23
title: Approval card on the session row
status: todo
depends: [AV-21, AV-22]
attended: false
---

# AV-23 — Approval card on the session row

**What to build:** The row surface for inline approvals (spec §11.3). A session with a pending approval opens a card under its row showing exactly what is being approved, with Allow, Deny and Deny with reason.

- [ ] `Edit`/`Write`: compact coloured diff of the real change, monospace, scrollable, capped height. `Bash`/`PowerShell`: full command, wrapped, risky span from AV-22 highlighted, cwd shown. Other tools: key arguments as a short list
- [ ] Allow / Deny / Deny with reason (inline text field, Enter sends, last 3 reasons offered)
- [ ] `destructive` tier: Allow is press-and-hold for 1s with a visible fill; a click does nothing
- [ ] Countdown to release (thin bar) so the user knows when the terminal takes over
- [ ] A waiting row with a pending approval auto-expands its card; this overrides "one row expanded at a time" only for cards
- [ ] Several pending on one session stack oldest first
- [ ] Card closes the moment the request resolves from anywhere (decided, released, answered in terminal)
- [ ] Peep and name still focus the terminal
- [ ] Vitest: diff building, argument formatting, hold-to-confirm timing; `?demo` gains one card of each tier

**Done when:** the owner allowed an edit, denied a command with a reason that Claude then acted on, and hold-confirmed a destructive command, all without touching the terminal.

Decide: diff rendering (recommended: a small in-repo line differ over `old_string`/`new_string`; no dependency).
