---
name: coderot
description: "Coderot: break code on purpose in a throwaway git worktree to find what the tests miss. Use when the user runs /peep:coderot."
disable-model-invocation: true
argument-hint: "<area or path prefix> [mutant cap]"
label: Coderot
steps: [setup, baseline, rot, honesty, cleanup, report]
---

# /peep:coderot: can the tests fail?

Facts and survivors, not reassurance. The owner's working tree is never edited; every mutation happens in a temporary worktree. Each heading names its step (`peep step:`, preamble).

## 1. Setup (step `setup`)

Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count).

- No `test`: reply `peep: no test command in .peep/setup.md; Coderot needs one.` and stop.
- Target: the first argument names an `area.<name>` (use its prefixes) or is a path prefix. No argument: the hot spot, the area whose prefixes the files of `git log -20 --name-only --format=` fall under most often (ties: setup order). Say `target: <area> (hot spot, <n> of the last 20 commits' files)` before step 2. No area matched: reply `peep: name an area: <area names>` and stop.
- Cap: the second argument if it is a number (max 50), else 20 mutants. Tier 1 and tier 2 together never exceed it.

## 2. Worktree and baseline (step `baseline`)

Create the worktree, install dependencies and run the baseline three times per `references/worktree.md`.
From the moment the worktree exists, every way out of this skill (done, error, cap hit, baseline red or flaky) goes through step 5 before step 6.

## 3. Rot (step `rot`)

Units, stubs, flips and what counts as caught: `references/rot.md`.

- Tier 1: stub whole units in the target. A stub no test notices is a survivor.
- Tier 2: small flips, only inside units whose tier 1 stub was caught.
- Edit files only under the worktree path, never the owner's tree. Reset the file after each mutant.

## 4. Test honesty (step `honesty`)

Read-only checks on the target and its tests: `references/honesty.md`. Flaky comes from the baseline runs.

## 5. Cleanup, always (step `cleanup`)

```
git worktree remove --force "$WT"
git worktree prune
git worktree list
```

`git worktree list` must show only the main tree and `$WT` must be gone. If the folder survives (a locked file), delete it and prune again.

## 6. Output (step `report`)

The table and ticket offer exactly per `references/output.md`, nothing before the header line:

```
Coderot: <target>, <n> mutants (cap <cap>), <s> survivors, HEAD <short sha>
| # | kind | where | what slipped through |
```

The cleanup line comes after the table: `worktree removed`, or `worktree left at <path>: remove it with git worktree remove --force`.
Write tickets only on "yes" or named rows; anything else writes nothing. The run ends on a `next` line (`references/output.md`).

## 7. Last step: run count

Per the preamble: `Why: a test that cannot fail proves nothing.`

Not here: mutating the owner's tree, uncommitted changes (Coderot rots HEAD), coverage tools, fixing the tests.
