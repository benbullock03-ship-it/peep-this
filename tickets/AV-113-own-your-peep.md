---
id: AV-113
title: Own your peep - colour and accessory override
status: open
depends: []
---

# AV-113 — Own your peep: colour and accessory override

**What to build:** Let a user pick their project's peep. The cwd hash stays the default; the override is per machine.

- [ ] Picker on the open row: the 5 palette colours and the unlocked accessories, live preview on the 96px stage
- [ ] Stored in widget settings keyed by the cwd hash; "reset" returns to the hash
- [ ] Applies to the row, strip, subagent children (colour only), toast icon and goose page
- [ ] Accessory list comes from one source so AV-115 can add unlocked ones
- [ ] Vitest for override resolution; Shoot both themes

**Done when:** a changed peep survives a restart and shows everywhere that project appears, and reset restores the hashed look; the owner confirms.
