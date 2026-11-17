---
id: AV-133
title: Moods off by default except clash
status: doing
depends: []
---

# AV-133: Moods off by default except clash

**What to build:** clash stays on, because it teaches "two agents on one file". The other moods (tired, worried, cheering, greeting, napping) go behind one setting that is off by default (AV-131 answer 3).

- [x] One moods setting, off by default and saved like the other settings
- [x] With it off, only the clash emote shows. The context nudge's tiredness counts as a mood.
- [x] With it on, behaviour is unchanged from AV-69 to AV-71
- [x] A test for the mood gate: clash passes when the setting is off, and the others are suppressed

**Done when:** on a fresh install only clash emotes appear, and switching the setting on brings the other moods back in `npm run tauri dev`.

## Checks

1. pass: setting `moods` in the widget `settings.json`, off by default (`mood.rs` `saved_moods_off_by_default_and_round_trips`; `moods_on`/`set_moods_on` commands; "Show moods" switch in Settings).
2. pass: off passes only startled; yawn (and so the context nudge) is suppressed (`mood.rs` `gate_passes_only_clash_when_moods_are_off`, `store.rs` `moods_off_by_default_leave_only_clash`).
3. pass: on is unchanged: existing mood, store and server mood tests pass with moods switched on (`cargo test`: 429 passed).
4. pass: Settings switch wired to the backend (`npm run build` type-checks `src/main.ts`, `src/settings.ts`).
5. owed to the owner: fresh install shows only clash emotes, and switching on brings the others back, in `npm run tauri dev`.

## Owed before done

- Owner check in `npm run tauri dev`: with no `moods` key in `%APPDATA%` widget `settings.json` only clash emotes show; Settings > "Show moods" on brings sweat, yawn plus nudge, worry, cheer, wave and nap back.

## Decisions taken without a ruling

- Tweet and rate-limit sweat (AV-81) count as moods and are off too ("only the clash emote shows"); the limit still takes the bubble's verb slot and the header pressure.
- With moods off the context nudge (Pack up / Not yet) goes too, because it is the yawn's bubble; Pack Up stays on the open row's tray.
- Gate sits in `store.rs` `refresh_moods` over `mood::derive`'s winner, so the snapshot never carries a suppressed mood; clash outranks every mood, so gating the winner never hides a clash.
- Demo mode (`npm run dev`) keeps showing its fixed demo moods; the demo switch is in-memory only.
