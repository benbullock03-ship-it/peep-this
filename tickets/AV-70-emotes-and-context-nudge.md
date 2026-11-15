---
id: AV-70
title: Emotes in the bubble and the context nudge
status: open
depends: [AV-63, AV-67, AV-69]
attended: true
---

# AV-70 — Emotes and context nudge

**What to build:** Moods show as emotes in the speech bubble, per the AV-63 emote sheet.

- [ ] Emote per mood, stepped animation, instant under reduced motion
- [ ] Hidden while an attention state holds the bubble
- [ ] Tired from the nudge threshold (setting 50 to 95 in steps of 5, default 80; owner ruling 2026-11-15) offers a Pack Up Jump (`/peep:packup`, reusing AV-67) in the bubble per the design; a second nudge once at 95%
- [ ] Vitest for mood to emote mapping and attention hiding; Shoot each emote in both themes

**Done when:** the owner has seen tired, worry and nap on live sessions in `npm run tauri dev`.
