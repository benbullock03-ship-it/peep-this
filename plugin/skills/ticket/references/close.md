# /peep:ticket close <id>: the owner's confirmation

`closed` means the owner checked the work. Claude never closes a ticket on its own judgement.

## 1. The owner's yes

- The owner typed `/peep:ticket close <id>` (or pasted it from the widget) in this session: that is the yes.
- Claude reached this step itself (a batch, its own suggestion, another skill): ask `Close <id>? Reply "yes".` and wait. Only a plain yes goes on.
- Status is not `done`: say which status it has and ask `Close <id> anyway? Reply "yes".` Only a plain yes goes on.
- Already `closed` or already in the archive: say so and stop.

## 2. Close

1. Set `status: closed`. If it was parked, delete `reason` and `wake`.
2. Archive it: create `<archive>` if missing, then `git mv <tickets><file> <archive><file>`.
3. Area note (F§3), one only:
   - Find the area from the ticket's commits: `git log --name-only --format= --grep="^<id>:"`. The area most of those files fall in wins (first matching prefix per file, F§1); a tie goes to the area listed first in the setup. No commits: use the paths the ticket names. No area matches: skip this step and say so in `owed`.
   - Read `kb/<area>.md`, the ticket's `## Done` section and the commit diff stat.
   - Rewrite the note so it is true now: change or delete bullets that this ticket made wrong, add what a newcomer must know. Present tense, a few bullets per section. Never append a log line or a dated entry.
   - Set the last line to `last touched: <id>, <YYYY-MM-DD>`.
   - Note missing: create it with the F§3 skeleton, and add its line to `kb/INDEX.md` in setup order (create the index with `# Areas` if missing).
4. Run the setup's `index` command, if any.
5. Stage the moved ticket, the area note, `kb/INDEX.md` if changed and the index file, by name. Commit `<id>: close`.

Waking tickets parked on this one (`wake: ticket:<id>`) is Ducks' job; do not do it here.
