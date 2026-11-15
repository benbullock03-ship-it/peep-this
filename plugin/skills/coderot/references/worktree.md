# Worktree, dependencies, baseline

Run from the repo root. Each Bash call is a fresh shell, so carry `$WT` as a literal path.

## Create

```
WT=$(mktemp -d)              # OS temp dir, never inside the repo
git worktree add --detach "$WT" HEAD
```

The empty temp dir is the worktree itself, so `git worktree remove` deletes it and nothing is left to clean by hand.

Note the short sha (`git rev-parse --short HEAD`) for the report header.

## Dependencies

Install fresh inside the worktree with the repo's lockfile installer, e.g. `npm ci --prefix "$WT"` (or `pnpm install --frozen-lockfile`, `yarn install --frozen-lockfile`, `pip install -r`, run from `$WT`). Rust needs nothing; cargo builds into `$WT`'s own target dir.

Never link or junction the owner's `node_modules`, `target` or venv into the worktree: a forced remove can follow the link and delete the owner's copy.

## Baseline

Run the setup's `test` command from `$WT` three times, unmodified. Record per run: exit code, wall time, failing test names.

- All three pass: baseline green. Time limit per mutant = 3x the slowest run.
- Some pass, some fail: the tests that failed in some runs but not all are **flaky** (report them); skip rot, since a catch could be the flake. Still run the honesty checks.
- All three fail: baseline red. Skip rot and honesty checks; go to cleanup.

## Between mutants

`git -C "$WT" checkout -- <file>` resets the one mutated file. Before the next mutant, `git -C "$WT" status --short` shows no tracked changes.

## Cleanup

In SKILL.md step 5. It runs even when a step above failed.
