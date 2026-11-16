---
id: AV-91
title: Tweet Tweet skill - tweet board, live pings, lessons into area notes
status: done
depends: [AV-74]
---

# AV-91 — Tweet Tweet skill

**What to build:** Sessions say when they are stuck or see a possible conflict, so many can run side by side and learn from each other.

- [x] `/peep:tweet` posts to `.peep/tweets.md`: who (session), ticket, the question or conflict, files involved; format in `formats.md`
- [x] Pings a live peer with Claude Code's SendMessage when one is reachable; the board alone otherwise
- [x] Skills that edit code (Ticket, Big Peep) check the board at start for tweets touching their files
- [x] An answered tweet is folded into its area note's "Watch out" or "Why" when its ticket closes, then removed from the board (Ticket close step)
- [x] The widget never relays messages; it only shows a tweet emote (AV-94)
- [x] Why line per AV-74
- [x] Add the skill to Walk's Extras block with a use-when line

**Done when:** two sessions on one repo exchange a tweet about a shared file, and closing the ticket moves the answer into the area note and off the board.

## Decisions taken without a ruling

- Skill dir `plugin/skills/tweet/` (command `/peep:tweet`); sub-commands: post `<text>`, `reply <n> <answer>`, none lists. Detail in `references/post.md` and `references/fold.md`.
- Board format is formats.md §7: `## <n> · <ticket> · <open|answered>` blocks with flat `from`, `kind` (`stuck`/`clash`), `files`, `ask`, `reply` lines; `n` is highest on the board + 1.
- "Who" is a handle (the session's ticket id, else a short task name), not a session id: a session cannot reliably learn its own id, and the widget can attribute the AV-94 emote from the hook event of the session editing `.peep/tweets.md`.
- First reply sets `answered`; the poster does not confirm.
- Board is shared through the working tree and never committed or staged, like `.peep/handoff.md`; no `.gitignore` edit.
- Fold goes into the area of the tweet's files (may be a second area note on close); fold rule shared in `tweet/references/fold.md`, cited by `close.md`. Ticket-less (`-`) tweets fold when the poster lists the board and sees the reply. Empty board: file deleted.
- Ping: only with `ListAgents` + `SendMessage`; peers whose working folder is inside the repo, matched by ticket or area, else up to 3; one fixed-shape message, no retry.
- Board check at start: work.md §1.5 (little peeps inherit it via step 1); Big Peep §1.2 reads the board and names matching tweets in each brief.
- Why line habit: think before code (`a question on the board costs a minute; a clash found at commit costs the afternoon`).

## Done 2026-11-16

- Live check in a scratch repo (setup `area.app: src/`, tickets T-01 and T-02 both on `src/a.js`): this session (T-01) posted clash tweet 1 and pinged the live peer `r91-1c` found by `ListAgents`; that headless session (T-02) replied with `/peep:tweet reply 1`, which set `answered` and pinged back (received here as a cross-session message).
- A third headless session ran `/peep:ticket close T-01`: it folded the answer into `kb/app.md` under `## Watch out` with no tweet number or handle, deleted `.peep/tweets.md` (no tweets left), archived the ticket and committed `T-01: close` without staging the board.
- The peer must run in the same permission mode as the pinger, or its harness holds the ping for approval.
