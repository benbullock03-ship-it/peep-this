# Drift: the repo against the Pack Up note

Never fix, stage, commit, stash or restore anything here.

- `git branch --show-current`, `git log --oneline -1`, `git status --short`.
- Branch or short hash differs from the note's `Repo:` line: list `git log --oneline <note hash>..HEAD` (max 5) if the hash is still known.
- Files under "Uncommitted, mine" no longer changed (committed or reverted), or changed files the note does not list.
- Each ticket id the note names (in progress, parked, `Then`): its `status` now against the note's.
- No mismatch: one **Loose** line, `drift: none, repo matches the Pack Up note`.
