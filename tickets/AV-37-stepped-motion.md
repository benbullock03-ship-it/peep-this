---
id: AV-37
title: Stepped motion language
status: todo
depends: [AV-35]
attended: false
---

# AV-37 — Stepped motion language

**What to build:** One motion rule for the whole widget: transitions run in 4 to 6 discrete frames over about 160ms, like the peeps' 8 to 12 fps (spec §12.3). This ticket sets the tokens and converts what exists; later surface tickets use the tokens.

- [ ] CSS tokens for duration and `steps()` timing, defined once
- [ ] Row enter, row leave, row flash and expand/collapse use the tokens
- [ ] The waiting pulse stays a slow loop but steps between its levels
- [ ] A Rust helper resizes or moves the window in the same number of steps, for AV-38, AV-39 and AV-41
- [ ] `prefers-reduced-motion`: every transition is instant; the waiting row keeps its static amber outline
- [ ] No smooth easing left in `styles.css` apart from colour hover fades under 150ms

**Done when:** rows entering, leaving and expanding in `?demo` visibly step, and with reduced motion on nothing animates.
