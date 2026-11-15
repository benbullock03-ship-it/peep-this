# Peep This

An always-on-top desktop companion for new Claude Code users: a widget that shows every live session as a peep, shipped together with a skill pack that teaches a working rhythm.

## Language

### Product

**Peep This**:
The product: the widget and the skill pack, installed as one.
_Avoid_: AgentView (code working name only)

**Widget**:
The desktop window that shows sessions. It shows and hands off; it never types into a session or starts one.
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
Checking a web UI change yourself in a real browser, against pass/fail questions written before the screenshots.
_Avoid_: visual test, QA

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
