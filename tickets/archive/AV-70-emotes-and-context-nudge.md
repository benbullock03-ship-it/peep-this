---
id: AV-70
title: Emotes in the bubble and the context nudge
status: closed
depends: [AV-63, AV-67, AV-69]
attended: true
---

# AV-70 — Emotes and context nudge

**What to build:** Moods show as emotes in the speech bubble, per the AV-63 emote sheet.

- [x] Emote per mood, stepped animation, instant under reduced motion
- [x] Hidden while an attention state holds the bubble
- [x] Tired from the nudge threshold (setting 50 to 95 in steps of 5, default 80; owner ruling 2026-11-15) offers a Pack Up Jump (`/peep:packup`, reusing AV-67) in the bubble per the design; a second nudge once at 95%
- [x] Vitest for mood to emote mapping and attention hiding; Shoot each emote in both themes

**Done when:** the owner has seen tired, worry and nap on live sessions in `npm run tauri dev`.

## Done 2026-11-15

`src/emote.ts` (`MOOD_EMOTE`, `emoteFor`, `nudgeFor`, `nudgeText`); `RowView.emote`/`nudge` in `src/rowView.ts`; `setEmote`, the nudge buttons and per-session snooze in `src/rows.ts`; emote and nudge styles in `src/styles.css`; sheets copied to `src/assets/emotes/`; `Session.mood` in `src/model.ts`; "Context nudge" slider in `src/settings.ts`, wired in `src/main.ts`. Backend: `mood::valid_nudge`, `load_nudge`, `nudge_pct`/`set_nudge_pct` commands (`nudgePct` in the widget's settings.json); `Store::set_nudge_pct` rederives moods, `refresh_moods` uses it instead of the constant. Demo: seven mood rows (`MOODY` in `src/demo.ts`) plus a hidden worry on sync-05. Tests: `src/emote.test.ts` (mapping, attention hiding, nudge, snooze, 95%), `nudge_setting_is_50_to_95_in_fives_and_round_trips`, `nudge_setting_moves_the_yawn_live`. `npm run build` ok, `npm test` 204 pass, `cargo test` 359 + 17 pass. Spec §13.5 updated.

Shoot (`?demo`, 360 and 1280, dark and light, reduced motion on and off): all six emotes at 16x16 in the trailing slot, right sheet per theme, frame 1 static under reduced motion; 0 emotes in attention rows; nudge bubble dashed in `--ps-accent` (#f7d64a / #805300); Pack up contrast 14.5 / 14.9, Not yet 6.8 / 6.2; Not yet hides the nudge and keeps the yawn; the last nudge shows Pack up only; no page overflow.

## Owed before closed

- The owner's live check in `npm run tauri dev`: tired, worry and nap on real sessions, and the slider moving the yawn.

## Decisions taken without a ruling

- Emote frames run at 8 fps per `emotes.json`, not the 160ms transition rule: they are loops, not transitions.
- Sheet by the bubble's fill: the app's dark theme has dark bubbles, so it takes `emotes-on-dark.png` (light ink), the reverse of the design's pairing.
- Sparkle and wave play twice and then hide, while the mood still holds (cheer 8s, greet 5s), per `emotes.json`.
- Pack up and Not yet sit under the plank, not inside the bubble: the closed row is one `<button>` and cannot hold buttons.
- Pack up gives its feedback on the button ("Copied" 1.2s, or "Type /peep:packup"), since the tray's hint bar is only on the open row; the open row shows both this and the tray's Pack up.
- Not yet snoozes per session in memory until 95% or until the yawn clears; it does not survive a restart.
- The last nudge at 95% stays while the yawn holds and has no Not yet, as drawn.
- Per-mood switches and the "nudge again at 95%" checkbox from the settings design are not built; the ticket asks only for the threshold.
- The nudge setting lives in the widget's settings.json next to `awayAfterMin`, reusing `away.rs`'s read and save helpers.
