---
id: AV-16
title: Bundled fonts and visual polish pass
status: todo
depends: [AV-04, AV-14, AV-15]
attended: true
---

# AV-16 — Bundled fonts and visual polish pass

**What to build:** Fredoka, IBM Plex Sans and IBM Plex Mono bundled locally (no Google Fonts at runtime), and a pass making the widget match `design/peep-this.dc.html`'s monitor section at widget scale.

- [ ] Fonts self-hosted under `src/assets/fonts`, OFL licences included
- [ ] Spacing, borders and colours from the design tokens in `src/styles.css`
- [ ] Row hover state; entrance animation for new rows; motion kept subtle
- [ ] Screenshots at 100% and 150% scaling reviewed with the owner

**Done when:** the owner signs off on the look.
