---
id: AV-72
title: Owner walkthrough - the checks only a human can do
status: closed
depends: [AV-67, AV-68, AV-73]
attended: true
---

# AV-72 — Owner walkthrough

**What this is:** every check owed to the owner from AV-57 to AV-68, cut to what Claude can't do headless (the GUI, the interactive question widget, a real install on your profile). Claude has run everything else in scratch repos; that evidence is in each archived ticket. One session, about 15 minutes, in the throwaway repo `F:\Projects\peep-trial` (two tickets, TR-01 and TR-02, no setup yet).

1. `npm run tauri dev` here, open the panel, click **Install integration**. In a new terminal: `cd F:\Projects\peep-trial`, `claude`, type `/peep:`. Ten commands listed (coderot, ducks, hello, packup, qt, reel, shoot, ticket, unpack, walk)? (AV-62)
2. Run `/peep:ducks` first: it replies that there's no repo setup and points to `/peep:walk`? Run `/peep:walk`. The questions come in batched rounds in the question widget, recommended answer first; accept, say yes. `.peep/setup.md` written? (AV-77)
2b. Run `/peep:ducks` again. In the widget: the ducks floppy goes in on the plank and comes out when it finishes? (AV-58, AV-64, AV-66)
3. Open that row. The shelf lists TR-01 and TR-02? Click TR-01: "Copied" moment shows and the terminal comes to the front; Ctrl+V, Enter. While it runs, the bubble reads "TR-01: Add a subtract function · ticketing" and the TR-01 chip moves todo → active → review on its own? (AV-64, AV-65, AV-67, AV-68)
4. Click the Question Time floppy in the tray, paste `/peep:qt let it divide too`, answer the rounds, say yes. The new ticket appears on the shelf? (AV-59, AV-65, AV-67)
5. Quit the widget from the tray and send any prompt in the session: no `hook error` lines? Start the widget again. (AV-73)
6. Click **Uninstall integration**, start a new `claude`: `/peep:` gone? (AV-62)

**Done when:** the owner replies "all good" or names the step that failed. A failed step becomes its own ticket. `F:\Projects\peep-trial` can then be deleted.

## Decisions taken without a ruling

- Steps 1 and 2 updated to today's pack before the walkthrough: AV-75 to AV-77 added reel, unpack and walk (ten commands, not six), and since AV-77 Ducks with no setup points to `/peep:walk` instead of writing the setup. Walk has no floppy until AV-84, so the floppy check moved to a second Ducks run (2b).
- Prep checked 2026-11-15: `F:\Projects\peep-trial` is clean at 6bb93c7, has TR-01 and TR-02, and has no `.peep/`.

## Done 2026-11-15

The owner ran steps 1 to 6 in `F:\Projects\peep-trial` and replied "all works great!". No step failed, so no new tickets.
