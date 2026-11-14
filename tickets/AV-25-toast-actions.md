---
id: AV-25
title: Allow and Deny buttons on the waiting toast
status: todo
depends: [AV-21, AV-22]
attended: false
---

# AV-25 — Allow and Deny buttons on the waiting toast

**What to build:** The waiting toast (spec §3.6) gains action buttons for low-risk requests (spec §11.3), so a `safe`-tier approval is one click from wherever the user is.

- [ ] `safe` tier: toast shows tool and target plus Allow and Deny. `shell` and `destructive`: no buttons; body says to open the inbox (hotkey shown)
- [ ] Buttons route to `decide_approval`; clicking the toast body still focuses the terminal
- [ ] A toast whose request already resolved does nothing when clicked, and is removed from the Action Center if the API allows
- [ ] De-duplication rules from AV-14 unchanged

**Done when:** the owner allowed a file read from a toast while another app was focused, and saw that a shell command's toast offered no buttons.

Decide: does `tauri-winrt-notification` deliver button activations in-process without a COM activator? If not, recommended: drop this ticket and rely on the hotkey inbox.
