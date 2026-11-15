---
id: AV-72
title: Owner walkthrough - the checks only a human can do
status: open
depends: [AV-67, AV-68]
attended: true
---

# AV-72 — Owner walkthrough

**What this is:** every check owed to the owner from AV-57 to AV-68, cut to what Claude can't do headless (the GUI, the interactive question widget, a real install on your profile). Claude has run everything else in scratch repos; that evidence is in each archived ticket. One session, about 15 minutes, in the throwaway repo `F:\Projects\peep-trial` (two tickets, TR-01 and TR-02, no setup yet).

1. `npm run tauri dev` here, open the panel, click **Install integration**. In a new terminal: `cd F:\Projects\peep-trial`, `claude`, type `/peep:`. Six commands listed? (AV-62)
2. Run `/peep:ducks`. The questions come as one batch in the question widget, recommended answer first; accept, say yes. `.peep/setup.md` written? In the widget: the ducks floppy goes in on the plank and comes out when it finishes? (AV-58, AV-64, AV-66)
3. Open that row. The shelf lists TR-01 and TR-02? Click TR-01: "Copied" moment shows and the terminal comes to the front; Ctrl+V, Enter. While it runs, the bubble reads "TR-01: Add a subtract function · ticketing" and the TR-01 chip moves todo → active → review on its own? (AV-64, AV-65, AV-67, AV-68)
4. Click the Question Time floppy in the tray, paste `/peep:qt let it divide too`, answer the rounds, say yes. The new ticket appears on the shelf? (AV-59, AV-65, AV-67)
5. Click **Uninstall integration**, start a new `claude`: `/peep:` gone? (AV-62)

**Done when:** the owner replies "all good" or names the step that failed. A failed step becomes its own ticket. `F:\Projects\peep-trial` can then be deleted.
