---
id: AV-130
title: Make it yours page
status: open
depends: [AV-126]
---

# AV-130 — Make it yours page

**What to build:** One settings page in the panel, per `design/make-it-yours/` (Question Time 2026-11-16).

- [ ] Peep look: the AV-113 colour and accessory override with a live 96px preview
- [ ] Alerts: toast on or off, the AV-111 sound switch, theme
- [ ] Pack for this repo (the open row's repo): each skill on or off (`skills.off`), `qt.round`, `shoot.widths`
- [ ] The backend writes only those preference keys in `<repo>/.peep/setup.md`, keeping every other line and key byte for byte; a key at its default is removed, as Walk does. No setup file: the section shows one line and a Jump for `/peep:walk`, and writes nothing
- [ ] Existing settings move here; the old settings view goes. Walk's tour names the page
- [ ] `cargo test`: setup round trip (unknown keys and body kept, default removes the key, missing file writes nothing). Shoot `?demo` both themes

**Done when:** switching a skill off on the page adds it to `skills.off` in that repo's setup with the rest of the file unchanged, the peep preview follows the pickers, and the owner confirms in `npm run tauri dev`.
