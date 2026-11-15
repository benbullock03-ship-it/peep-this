---
id: AV-58
title: Ducks skill - repo check, first-run setup, batch close, wake parked tickets
status: closed
depends: [AV-56, AV-57]
attended: false
---

# AV-58 — `/peep:ducks` ("Get your ducks in a row")

**What to build:** The present-tense check of a repo.

- [ ] First run with no repo setup: a batched round of questions (prefix, commands, areas, all with guessed recommended answers), read-back, then write the setup
- [ ] Reads `.peep/handoff.md` from the last Pack Up first, if present
- [ ] Every run, gathered cheaply (git status, the ticket index, one test run): `doing` tickets with no recent commits, `done` tickets waiting for the owner, parked tickets, failing tests, uncommitted or unrelated work, rules in `CLAUDE.md` likely to bite the next change
- [ ] Wakes parked tickets whose wake-up condition is met (referenced ticket closed, date passed); free-text conditions are listed for the owner, not judged silently
- [ ] Batched questions about anything the owner must decide, including "close these done tickets?"
- [ ] Read-back, then a tidy list or "clear to proceed"; nothing written before the yes

**Done when:** on a test repo with each kind of loose end, Ducks finds all of them, asks once, and acts only on the confirmed items.

## Done

`plugin/skills/ducks/SKILL.md` (60 lines) plus `references/setup.md`, `gather.md`, `questions.md`, `act.md`. Headless proof (2026-11-15, `claude --plugin-dir plugin -p "/peep:ducks" --permission-mode acceptEdits`, git/npm/node Bash allowed) on two scratch repos:

- Loose-ends repo: proposed wake DT-04 (`ticket:DT-01`, archived) and DT-05 (past `date:`); close DT-03 (`done`); kept DT-06 free-text wake for the owner; DT-07 future date not met; DT-02 `doing` with no commit in 7 weeks; failing test named; `notes.txt` flagged not mine from the handoff, `src/core/b.js` unrelated; 3 `CLAUDE.md` rules; next `/peep:ticket DT-08` from the handoff. No question tool headless, so it gave the read-back marked `(unconfirmed)` and stopped. File hashes and `git status`/`git log` identical before and after.
- No-setup repo: guessed prefix `WX`, `npm test`, `npm run build`, dev/url, areas api and ui; read-back; wrote nothing.
- Act after yes (fresh copy of the loose-ends repo, then `claude -c -p` twice): the answers ("close DT-03, DT-02 back to open, DT-06 not yet") came with a correction ("do not wake DT-05"). That produced a fresh, complete read-back with DT-05 moved to **Loose**, and nothing written. Then "yes": three commits, `DT-04: wake (DT-01 closed)` (status open, reason/wake removed), `DT-02: doing -> open` and `DT-03: close` (`git mv` to archive, status closed). DT-05/06/07, `notes.txt`, `src/core/b.js` and `.peep/handoff.md` untouched. Area note skipped and reported: DT-03 had no commits and named no paths, as `close.md` says. The index command ran and was a no-op.
- Fixed from these runs: the next ticket now goes under **Watch** (a stray `Next:` line had appeared outside the groups), and the output header says `acted on` instead of `done`.

## Decisions taken without a ruling

- First run continues into the check after writing the setup, in the same run.
- `.peep/setup.md` is written but not committed or staged; the owner decides.
- "No recent commits" for `doing` means no commit naming `<id>:` in 7 days and no uncommitted edit to its ticket file.
- Wakes whose condition is met go straight to the read-back's **Do** without a question; the owner can strike one there.
- One AskUserQuestion round, at most 4 questions in fixed priority (close, free-text wakes, stale doing, handoff rulings, broken parked); overflow is listed as `undecided` for the next run.
- Each wake commits as `<id>: wake (<condition>)`; the index is regenerated once at the end and committed as `ducks: index`.
- A ticket parked on one closed in the same run is listed as ready to wake next run, not woken unconfirmed.
- Ducks never parks a ticket itself; a stale `doing` can only go back to `open` or stay, parking goes through `/peep:ticket park`.

## Owed before done

- Owner runs `/peep:ducks` interactively and checks the AskUserQuestion round renders as one batch, recommended first. Headless runs only proved answers given as text.
- Owner runs it on a repo with no `.peep/setup.md`: answer the setup round, say yes, check `.peep/setup.md` matches the read-back.
- A close that rewrites an area note (a ticket with commits in an area) has not been exercised through Ducks.
