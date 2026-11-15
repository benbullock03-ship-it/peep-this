# Rot

## Picking units

Source files under the target prefixes, not test files, fixtures, generated code or type-only files. A unit is an exported or public function, method or class constructor. Order: files changed most recently first (`git log --format= --name-only -- <prefixes>`), then the rest alphabetically. Stop picking when the cap is reached.

## Tier 1: stub the unit

Replace the unit's body with the cheapest thing that still compiles:

| Returns | Stub |
|---|---|
| nothing / void / `()` | empty body |
| bool | `false` (if `false` was already the obvious result, `true`) |
| number | `0` |
| string | `""` |
| array / Vec / list | empty |
| object, Option, nullable | `undefined` / `None` / `null` |
| Result | `Ok(Default::default())` when it compiles, else `todo!()` |
| anything else (TS) | `return undefined as any` |

A stub that fails to compile or type-check is **not** a catch; note it as `skipped: won't compile` and do not count it against the cap.

## Tier 2: small flips

Only inside units whose tier 1 stub was caught. One flip per mutant, at most 3 per unit, starting with the branches no test name hints at:

- negate an `if`/`while` condition, or force it `true`/`false`
- `<` to `<=`, `>` to `>=`, `==` to `!=` (and back)
- off by one: `+ 1` to `+ 0`, `length - 1` to `length`
- early return: return the tier 1 stub value as the unit's first line inside a branch
- dropped await: remove one `await` (TS/JS) or `.await`-adjacent `?` (Rust)
- `&&` to `||`

## Caught or survived

Run the setup's `test` from `$WT` after each mutant.

- Non-zero exit, or killed at the time limit: **caught**.
- An equivalent mutant (same result for every input, e.g. `n < lo` -> `n <= lo` where both branches return `lo` at `n == lo`) is no test's fault: drop it, pick another flip, and do not count it.
- Exit 0: **survivor**. Record file:line, tier, and the one-line change (e.g. `if (n > limit)` -> `if (!(n > limit))`).
