---
id: AV-71
title: Clash when two sessions edit the same file
status: open
depends: [AV-54, AV-63, AV-69, AV-70]
attended: true
---

# AV-71 — Clash

**What to build:** The ambient clash mood from ADR 0002.

- [ ] Backend: two different sessions edit the same absolute path within 5 minutes (from AV-54's edit payloads); both get the startled mood with the file name; expires after a short hold
- [ ] Frontend: both bubbles show the startled emote and file name, linked per the AV-63 clash design
- [ ] No warning, no blocking, no trail entry
- [ ] `cargo test`: same file within window, outside window, same session twice (no clash), three sessions

**Done when:** two live sessions editing one file show the clash in `npm run tauri dev` and the owner confirms it.
