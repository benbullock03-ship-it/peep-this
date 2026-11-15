# Output and ticket offer

## Table

Rank, worst first. Within a kind, tier 1 before tier 2, then by file path.

1. `survivor t1`: a whole unit can go and no test notices
2. `survivor t2`: a logic flip no test notices
3. `only`
4. `flaky`
5. `skipped`, `commented-out`
6. `hollow`
7. `untested change`

Columns: `#`, `kind`, `where` (`path:line`, or `path > test name`), `what slipped through` (one line: the mutant's change, or why the test is dishonest). No other columns, no prose between rows. Nothing found: one row `| - | clean | <target> | no survivors, no honesty findings |`.

Below the table, at most two lines: `skipped: <n> mutants won't compile`, and `not rotted: uncommitted changes` when `git status` was dirty. Omit lines that would say zero or nothing. Then the cleanup line from SKILL.md step 5.

## Ticket offer

When there is at least one `survivor`, `only` or `flaky` row, end with exactly:

`Turn the top <k> into tickets? Reply "yes", or name the row numbers.`

`k` is the number of such rows, at most 5. Nothing after that line. Wait. Only "yes" or a list of row numbers writes; anything else replies `peep: nothing written.`

## Writing tickets (on yes)

- Read `prefix`, `tickets`, `archive` from setup. Format: `${CLAUDE_PLUGIN_ROOT}/references/formats.md` §2.
- Ids: next free numbers after the highest `<prefix>-NN` in `tickets` and `archive` together.
- One ticket per row: title `Test <unit or test name>: <what slipped through>`, `status: open`, `depends: []`. Body: one checklist item (a test that fails against that exact mutant, or the honesty fix), and `**Done when:** /peep:coderot <target> no longer lists this row.`
- If setup has `index`, run it once and check it exited 0.
- Reply only `peep: <n> tickets written` and one `- <id>: <title>` line each.
