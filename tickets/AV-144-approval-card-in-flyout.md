---
id: AV-144
title: Approval card in the strip flyout
status: open
depends: []
---

# AV-144: Approval card in the strip flyout

**What to build:** CLAUDE.md "Waiting while collapsed" says the approval card mounts in the flyout when the widget is in `strip`. It does not: `src/flyoutPage.ts` line 57 is only a comment, and the AV-137 live check (2026-11-17) found no card and no Allow button in strip mode or its flyout; it had to switch to panel to approve. Inline approvals are on the v1 cut line, and the widget launches in `strip`.

- [ ] A held `PermissionRequest` for a session shows its approval card in that row's flyout while the widget is in `strip`, with the same tiers as the panel card (`safe` one click, `shell` with the full command, `destructive` hold 1s)
- [ ] Deciding from the flyout goes through the same `decide_approval` path and logs to the trail
- [ ] Release rules unchanged: about 60s, or at once when that terminal is focused
- [ ] A test for mounting the card in the flyout page

**Done when:** in `npm run tauri dev` in `strip` mode, a held permission request flies out with its card, and a real click on Allow delivers `allow` to the hook.
