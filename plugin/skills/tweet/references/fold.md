# Fold an answered tweet into its area note

Used by Ticket's close step and by `/peep:tweet` list mode. Note format: F§3.

1. Area: the first `area.<name>` (F§1) matching the tweet's files; files in several areas take the one most of them fall in, a tie the one listed first. No setup or no match: remove the tweet without folding and say so (`owed` for Ticket, one line for Tweet).
2. Read `kb/<area>.md` (missing: create it with the F§3 skeleton and add its `kb/INDEX.md` line, as close.md §2.3 does).
3. Turn the ask and replies into what a newcomer must know, present tense:
   - a trap, a collision, a file two areas share: one bullet under `## Watch out`;
   - a reason something is the way it is: one bullet under `## Why it's like this`.
   - Already said: sharpen that bullet, do not add one. Not true any more: write nothing.
4. No tweet number, date, handle or "asked/answered" wording in the note. Set the last line to `last touched: <ticket>, <YYYY-MM-DD>` (ticket `-`: leave it as is).
5. Delete the tweet's block from the board. No tweets left: delete the file.
