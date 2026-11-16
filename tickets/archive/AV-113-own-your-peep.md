---
id: AV-113
title: Own your peep - colour and accessory override
status: closed
depends: []
---

# AV-113 — Own your peep: colour and accessory override

**What to build:** Let a user pick their project's peep. The cwd hash stays the default; the override is per machine.

- [x] Picker on the open row: the 5 palette colours and the unlocked accessories, live preview on the 96px stage
- [x] Stored in widget settings keyed by the cwd hash; "reset" returns to the hash
- [x] Applies to the row, strip, subagent children (colour only), toast icon and goose page
- [x] Accessory list comes from one source so AV-115 can add unlocked ones
- [x] Vitest for override resolution; Shoot both themes

**Done when:** a changed peep survives a restart and shows everywhere that project appears, and reset restores the hashed look; the owner confirms.

## Done 2026-11-16

Live check in `npm run tauri dev` (CDP 9333) on a throwaway session in a scratch repo, hashed look yellow/bow, key `0ffc2fe2`:

- Picked pink + crown in the open row's picker: stage 96px went pink/crown, `peepLooks` saved.
- Restarted the widget: row (48) and strip (40) still pink/crown.
- Strip mode, `Notification` hook POST: flyout peep pink/crown `waiting`; the waiting toast (wpndatabase id 16720) carries `appLogoOverride` `icons/0ffc2fe2.png`, drawn pink/crown.
- Reset in the picker: `peepLooks` empty, stage, strip and toast icon back to yellow/bow, Reset disabled.
- Picked again, ended the session: closed shelf entry pink/crown; reset (`set_peep_look` null) put it back to yellow/bow.
- Owner looked at the screenshots and said "looks good!".
- Children (colour only) not seen live; covered by the row tests.

## Decisions taken without a ruling

- A pick applies and saves at once, so the 96px stage is the live preview; no separate preview/confirm step.
- The picker sits behind a "Change peep" toggle after the cwd line on the open row, so the open row does not grow by default.
- Settings key is `peepLooks` in the widget `settings.json`, `{ <fnv1a cwd hash, 8 hex>: { color?, accessory? } }`; a field equal to the hashed one is dropped, and an empty look is removed, so picking the hashed look back is a reset.
- Accessories come from `pickerAccessories(unlocked)` in `src/identity.ts`: the free 14 now; AV-115 passes its unlocked ids. The backend checks only an accessory's shape, since the list lives in the frontend.
- Toasts had no peep icon; the widget now draws each live project's peep (`waiting` pose, 96px, no smoothing) and hands it to `set_toast_icon`, stored in `%APPDATA%/PeepThis/icons/<hash>.png` and used as the waiting toast's app logo. Limit toasts keep no icon.
- Goose page (owner ruled 2026-11-16 to do it here): `map.mjs` used its own 31-hash colour and no accessory. It now uses the widget's FNV-1a identity (`peepLook`) overridden by `peepLooks` read from `%APPDATA%/PeepThis/settings.json` (`readLooks`); pins match `src/identity.test.ts`. The skill's own voice peeps (panels, top pick) stay the yellow house peep.
- The closed shelf and the flyout window follow the override too ("everywhere that project appears"); the flyout follows via the `peep-looks` event.
