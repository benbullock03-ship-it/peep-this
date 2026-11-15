# First run: write the repo setup

The repo has no `.peep/setup.md`, or it has no `prefix`. One batched round, a read-back, then write it. Nothing is written before the yes.

## 1. Guess first (do not print)

Read only what exists:

- Ticket folder: `tickets/`, else `docs/tickets/`, else none (default `tickets/`). Prefix: the most common `^[A-Z]+-[0-9]+` in its filenames; else 2 to 4 capitals from the repo folder name.
- Commands: `package.json` scripts (`test`, `build`, `dev`, and `tickets` or `index` for the index), using `npm run <script>` (`npm test` for test); `Cargo.toml` (`cargo test`, `cargo build`; prefix `cd <dir> && ` if not at the root); `pyproject.toml`/`pytest.ini` (`pytest`); `go.mod` (`go test ./...`, `go build ./...`); `Makefile` targets `test`/`build`. Several stacks: join with ` && `, frontend first.
- `url`: only when there is a `dev` command; the port from the Vite/Next config or the dev script, else `http://localhost:5173`.
- Areas: the top-level folders under `src/` (or the source roots when there is no `src/`), at most 5, each `area.<folder>: <path>/`. Lowercase, hyphens only.

## 2. One round

One AskUserQuestion call, these 4 questions, guess first with ` (recommended)`:

| header | question | options |
|---|---|---|
| `Prefix` | Which ticket id prefix should this repo use? | the guess; one other plausible prefix |
| `Test` | Which command runs the tests? | the guess; "No test command" |
| `Build` | Which command builds or type-checks? | the guess; "No build command" |
| `Areas` | Which areas should area notes cover? | the guessed list, comma-separated; "No areas yet" |

Each option's description says where the guess came from (e.g. "package.json scripts.test"). No guess for a question: offer "None for now (recommended)" and one example. If no answers come back, give the read-back with the guesses marked `(unconfirmed)` and stop.

## 3. Read-back

Per F§4, groups **Tickets** (prefix, folder, archive, index command), **Commands** (build, test, dev, url), **Areas** (one per line). A correction gets a fresh, complete read-back.

## 4. Write on yes

Create `.peep/` and write `.peep/setup.md` in the shape of `${CLAUDE_PLUGIN_ROOT}/references/examples/.peep/setup.md`: only keys with a value, `tickets` on its own line with a trailing `/`. Body: `# Repo setup`, then `Written by /peep:ducks on <YYYY-MM-DD>. Edit by hand if a command changes.`

Do not commit or stage it; the owner decides. Mention it under `loose:` in the output as `.peep/setup.md new, uncommitted`.
