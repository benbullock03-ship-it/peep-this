---
id: AV-24
title: Keyboard approval inbox on a global hotkey
status: todo
depends: [AV-23]
attended: false
---

# AV-24 — Keyboard approval inbox on a global hotkey

**What to build:** A global hotkey that brings the oldest pending approval across all sessions front and centre, answerable without the mouse or a window switch (spec §11.4). It reuses AV-23's card.

- [ ] Global hotkey registered through Tauri's global-shortcut plugin; configurable; a clash with another app is reported, not silent
- [ ] Hotkey shows the widget if hidden, focuses it, and puts the oldest pending card in focus with session name, peep and project colour
- [ ] Keys: `Y` allow, `N` deny, `R` deny with reason, `Tab`/`Shift+Tab` next/previous, `Enter` focus that session's terminal, `Esc` dismiss and return focus to the previous window
- [ ] `destructive` tier: `Y` must be held for 1s
- [ ] After a decision the next pending card takes focus; when none remain, focus returns to the window the user came from
- [ ] Hotkey with nothing pending flashes the header and does nothing else
- [ ] Header shows a pending count when > 0

**Done when:** the owner cleared three pending approvals from three sessions using only the keyboard and ended up back in the window they started in.

Decide: default hotkey (recommended: `Ctrl+Alt+I`; `Ctrl+Alt+P` is the panel/strip toggle since the surface grill, AV-40). Decide: a separate centred overlay window vs focusing the docked widget (recommended: focus the docked widget; one surface).
