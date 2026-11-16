---
id: AV-113
title: Own your peep - colour and accessory override
status: doing
depends: []
---

# AV-113 — Own your peep: colour and accessory override

**What to build:** Let a user pick their project's peep. The cwd hash stays the default; the override is per machine.

- [x] Picker on the open row: the 5 palette colours and the unlocked accessories, live preview on the 96px stage
- [x] Stored in widget settings keyed by the cwd hash; "reset" returns to the hash
- [ ] Applies to the row, strip, subagent children (colour only), toast icon and goose page
- [x] Accessory list comes from one source so AV-115 can add unlocked ones
- [x] Vitest for override resolution; Shoot both themes

**Done when:** a changed peep survives a restart and shows everywhere that project appears, and reset restores the hashed look; the owner confirms.

## Owed before done

- Goose page: `plugin/skills/goose/scripts/map.mjs` draws every peep `yellow` with no project identity, so the override needs a plugin change (read `peepLooks` from `%APPDATA%/PeepThis/settings.json` by `cwd_key`). Plugin files were out of bounds for this run.
- Owner check in `npm run tauri dev`: pick a look, restart the widget, see it on row, strip, children (colour), closed shelf, flyout and a waiting toast's icon; Reset restores the hashed look.

## Decisions taken without a ruling

- A pick applies and saves at once, so the 96px stage is the live preview; no separate preview/confirm step.
- The picker sits behind a "Change peep" toggle after the cwd line on the open row, so the open row does not grow by default.
- Settings key is `peepLooks` in the widget `settings.json`, `{ <fnv1a cwd hash, 8 hex>: { color?, accessory? } }`; a field equal to the hashed one is dropped, and an empty look is removed, so picking the hashed look back is a reset.
- Accessories come from `pickerAccessories(unlocked)` in `src/identity.ts`: the free 14 now; AV-115 passes its unlocked ids. The backend checks only an accessory's shape, since the list lives in the frontend.
- Toasts had no peep icon; the widget now draws each live project's peep (`waiting` pose, 96px, no smoothing) and hands it to `set_toast_icon`, stored in `%APPDATA%/PeepThis/icons/<hash>.png` and used as the waiting toast's app logo. Limit toasts keep no icon.
- The closed shelf and the flyout window follow the override too ("everywhere that project appears"); the flyout follows via the `peep-looks` event.
