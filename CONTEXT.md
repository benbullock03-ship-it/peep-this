# Peep This

An always-on-top desktop companion for new Claude Code users: a widget that shows every live session as a peep, shipped together with a skill pack that teaches a working rhythm.

## Language

### Product

**Peep This**:
The product: the widget and the skill pack, installed as one.
_Avoid_: AgentView (code working name only)

**Widget**:
The desktop window that shows sessions. It shows and hands off; it never types into a session or starts one. It may end one, on a hold.
_Avoid_: tray, app, monitor

**Skill pack**:
The set of Peep skills delivered as one Claude Code plugin alongside the widget.
_Avoid_: Peep These Skills, toolkit

**Peep skill**:
One skill in the skill pack, invoked in a session under the pack's namespace.
_Avoid_: command, tool

### Fleet

**Session**:
One running Claude Code conversation on this machine.
_Avoid_: agent, instance, chat

**Peep**:
The pixel marshmallow that stands for one session and shows its state.
_Avoid_: pet, avatar, bot

**Jump**:
The widget's way of starting something: copy the command and focus that session's terminal.
_Avoid_: hand-off, launch, run, send

### Work

**Ticket**:
One unit of work in a repo's `tickets/` folder, named by id and title ("AV-34: item name").
_Avoid_: task, issue, item

**Ticket status**:
One of `open`, `doing`, `blocked`, `deferred`, `done`, `closed`. `done` is Claude's claim; `closed` is the owner's confirmation and the only status that archives.
_Avoid_: resolved, finished, complete

**Parked ticket**:
A ticket that is `blocked` or `deferred`. It always carries a reason and a wake-up condition, so it cannot be forgotten.

**Wake-up condition**:
What returns a parked ticket to `open`: another ticket closing, a date passing, or (as a fallback) a free-text note.
_Avoid_: trigger, reminder
_Avoid_: backlog, icebox

**Area note**:
A short living page for one area of the code: how it works now, what to watch out for, and why. Closing a ticket edits its area note rather than appending to it.
_Avoid_: log, ledger, journal, knowledge base entry

**Pack Up**:
Carrying a session forward into a new one: a short directive note, saved in the repo and copied to the clipboard, that opens the next session. When the next direction is unclear, Claude asks 2 or 3 questions first.
_Avoid_: handoff, summary, recap

### Skill pack

**Unpack**:
Opening a session from a Pack Up note: Ducks checks the repo still matches it, reads back any drift, then runs the note's `Then` on a yes. No longer a skill of its own (AV-103).
_Avoid_: resume, restore, load handoff, /peep:unpack

**Walk Through**:
The first-run skill: tours the skills in rhythm order, asks batched questions about the repo and how the owner works, and writes the repo setup with per-skill preferences.
_Avoid_: onboarding, tutorial, wizard

**North star**:
The one line in `.peep/reel.md` that says what the project is for, written by Walk Through with the hook and non-goals. Question Time and Ducks flag work that does not serve it, never block it.
_Avoid_: mission, goal

**Why line**:
The one line a skill adds about why its habit matters, shown for a skill's first 3 runs and then dropped.
_Avoid_: tip, hint, tutorial text

**Sneak Peep**:
UI prototyping before building: 2 or 3 throwaway variants served side by side; the picked one becomes the design reference.
_Avoid_: mockup, spike

**AI hallmark**:
A pattern that marks UI as AI-made by default (purple gradients, emoji headers, generic card grids, filler copy), listed in the pack's checklist.
_Avoid_: slop (fine in conversation, not in UI)

**Big Peep**:
The orchestrator skill that runs a batch of tickets, one little peep per ticket, and asks a stale little peep for its status without stopping it.
_Avoid_: manager, supervisor

**Little peep**:
A subagent working one ticket under Big Peep.
_Avoid_: worker, child agent

**Tweet Tweet**:
Sessions telling each other when stuck or seeing a possible conflict: a post on the tweet board, plus a live ping to a peer when one is reachable.
_Avoid_: chat, message bus

**Tweet board**:
`.peep/tweets.md`, the repo's open tweets. An answered tweet is folded into its area note when its ticket closes, then removed.
_Avoid_: log, inbox

**Duck Duck Goose**:
The report maker: writes a page for the owner under `.peep/graph/`, either the code map, repo health (hot spots, biggest files, untested areas, stale tickets, pattern cards, and the `candidates.json` the Learned list reads) or a one-off topic. Agents querying the graph is secondary; uses graphify when installed.
_Avoid_: knowledge graph (fine in docs), map

**Closed shelf**:
The widget's list of sessions that ended in the last 8 hours, flagging uncommitted changes, a ticket left `doing`, no Pack Up, or an ending in an attention state.
_Avoid_: history, recent, graveyard

**Arc line**:
One line per repo on the open row: north star, closed / open / parked ticket counts, last ticket closed.
_Avoid_: progress bar, roadmap

**Question Time**:
The forward-looking grill: batched rounds of questions about a new idea or problem, then a read-back, then tickets.
_Avoid_: grill, interview, brainstorm

**Ducks**:
The present-tense check of a repo: unclosed and parked tickets, failing tests, rules that bite, risks of regression. Ends in a read-back, then a tidy list or "clear to proceed".
_Avoid_: audit, status, standup

**Read-back**:
Question Time's or Ducks' short statement of what Claude now understands, given after the questions and before any ticket or action. Nothing is written until the owner confirms it or corrects it.
_Avoid_: summary, recap, plan

**Shoot**:
Checking a web UI change yourself in a real browser, against pass/fail questions written before the screenshots. `--scrub` adds a pass that scrubs the UI of AI hallmarks and messy spacing and text, reporting where each comes from with alternatives, and fixing only what the owner approves (AV-119).
_Avoid_: visual test, QA, lint, design review

**Repo setup**:
The per-repo record of conventions (ticket prefix, build and test commands, areas) that every Peep skill reads instead of guessing. Ducks writes it on first run.
_Avoid_: config, settings

**Coderot**:
The test-honesty check: deliberately rots code in a throwaway copy, whole units first and then small logic flips, and reports the rot the tests let through. It also flags hollow, skipped, flaky and missing tests.
_Avoid_: mutation testing (fine in docs, not in UI), test audit

**Floppy**:
The pixel-art disk that represents one Peep skill in the widget, labelled with the skill's name.
_Avoid_: icon, badge, cartridge

**Inserted floppy**:
The floppy sticker on a row's plank while that session is running the skill.
_Avoid_: skill badge, active skill indicator

**Floppy tray**:
The set of floppies on an open row; clicking one Jumps that skill's command to that session. A floppy comes out when the session's turn stops.
_Avoid_: launcher, menu

**Ticket shelf**:
The list of a repo's open and parked tickets on an open row; clicking one Jumps `ticket <id>` to that session.
_Avoid_: backlog view, board

**Context nudge**:
The peep's bubble suggesting Pack Up, then /clear, once a session's context passes the threshold. Shown as the peep's tiredness.
_Avoid_: warning, alert

### Moods

**Mood**:
A short-lived feeling a peep shows through a bubble emote on top of its state: tired (sweat drop, then yawn), worried, cheering, greeting, napping (zzz), or startled by a clash. A mood never changes the session's state, and an attention state always takes the bubble first.
_Avoid_: state, emotion, reaction (as a noun)

**Emote**:
The small pixel symbol in a speech bubble that shows a mood.
_Avoid_: emoji, icon

**Clash**:
Two sessions editing the same file within a few minutes of each other. Both peeps show a startled emote at each other and the bubble names the file. Ambient only: no warning and no blocking.
_Avoid_: collision, conflict, collision radar

**Survivor**:
A piece of rot that no test caught.
_Avoid_: escaped mutant, miss
