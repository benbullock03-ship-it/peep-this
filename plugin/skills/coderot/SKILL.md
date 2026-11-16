---
name: coderot
description: "Coderot: break code on purpose in a throwaway git worktree to find what the tests miss. Use when the user runs /peep:coderot."
disable-model-invocation: true
argument-hint: "<area or path prefix> [mutant cap]"
label: Coderot
---

# /peep:coderot: can the tests fail?

Facts and survivors, not reassurance. The owner's working tree is never edited; every mutation happens in a temporary worktree.

## 1. Setup

Read `.peep/setup.md` (`${CLAUDE_PLUGIN_ROOT}/references/formats.md` §1). If it is missing, reply with exactly this and stop:

`peep: no repo setup here. Run /peep:walk first, so Coderot knows the test command and areas.`

`skills.off` lists `coderot`: F§1 applies ("F§1" is `${CLAUDE_PLUGIN_ROOT}/references/formats.md` §1).

- No `test`: reply `peep: no test command in .peep/setup.md; Coderot needs one.` and stop.
- Target: the first argument names an `area.<name>` (use its prefixes) or is a path prefix. No argument: reply `peep: name an area: <area names>` and stop.
- Cap: the second argument if it is a number (max 50), else 20 mutants. Tier 1 and tier 2 together never exceed it.

## 2. Worktree and baseline

Create the worktree, install dependencies and run the baseline three times per `references/worktree.md`.
From the moment the worktree exists, every way out of this skill (done, error, cap hit, baseline red or flaky) goes through step 5 before step 6.

## 3. Rot

Units, stubs, flips and what counts as caught: `references/rot.md`.

- Tier 1: stub whole units in the target. A stub no test notices is a survivor.
- Tier 2: small flips, only inside units whose tier 1 stub was caught.
- Edit files only under the worktree path, never the owner's tree. Reset the file after each mutant.

## 4. Test honesty

Read-only checks on the target and its tests: `references/honesty.md`. Flaky comes from the baseline runs.

## 5. Cleanup (always)

```
git worktree remove --force "$WT"
git worktree prune
git worktree list
```

`git worktree list` must show only the main tree and `$WT` must be gone. If the folder survives (a locked file), delete it and prune again.

## 6. Output

The table and ticket offer exactly per `references/output.md`, nothing before the header line:

```
Coderot: <target>, <n> mutants (cap <cap>), <s> survivors, HEAD <short sha>
| # | kind | where | what slipped through |
```

The cleanup line comes after the table: `worktree removed`, or `worktree left at <path>: remove it with git worktree remove --force`.
Write tickets only on "yes" or named rows; anything else writes nothing.

## 7. Last step: run count

Bump `coderot` in `.peep/runs` (formats §1). If it was below 3, add a final line: `Why: a test that cannot fail proves nothing.`

Not here: mutating the owner's tree, uncommitted changes (Coderot rots HEAD), coverage tools, fixing the tests.
