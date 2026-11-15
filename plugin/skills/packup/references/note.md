# Pack Up note

`.peep/handoff.md` is plain text, pasted unedited as the next session's first message. These lines, in this order:

```
Pack Up note, <YYYY-MM-DD>. Carry on from here.
Repo: <path>, branch <name> at <short hash>. Nothing pushed. (or: pushed to <remote/branch>)
In progress: <ticket id> <title> (<status>), or "none"
Left to do on it: <one line, or "none">
Parked, touched this session: <id> (<status>): <reason>; wakes <wake>   one line each, or "none"
Uncommitted, mine: <files, or "none">
Uncommitted, not mine (do not touch): <files, or "none">
Checks: build <pass|fail|not run>, test <pass|fail|not run>. Failing: <what, or "none">
Owner rulings wanted: <one line each, max 3, with the recommended answer, or "none">
First command: <the exact command>
Then: /peep:ticket <next id>   (or /peep:ducks if no ticket is next)
```

## Rules

- Under 25 lines. If it is longer, cut, do not wrap.
- Directive only: what to run, what to finish, what not to touch. No history, no reasoning, no account of what was done; the tickets and `git log` hold that.
- A line with nothing to say reads "none", not dropped, so the reader knows it was checked.
- "Not mine" means changes this session did not make (another session's work); list them so nobody sweeps them into a commit.
- No push, release or deploy command unless the owner approved it this session; then it goes under "First command" with the word "approved".
- Repo without tickets: drop the ticket lines and say what to do next in "Left to do".
