# Walk: the rounds and the setup file

Two batched rounds, a read-back, then write. Nothing is written before the yes.

## 1. Guess first (do not print)

Read only what exists:

- Ticket folder: `tickets/`, else `docs/tickets/`, else none (default `tickets/`). Prefix: the most common `^[A-Z]+-[0-9]+` in its filenames; else 2 to 4 capitals from the repo folder name.
- Commands: `package.json` scripts (`test`, `build`, `dev`, and `tickets` or `index` for the index), using `npm run <script>` (`npm test` for test); `Cargo.toml` (`cargo test`, `cargo build`; prefix `cd <dir> && ` if not at the root); `pyproject.toml`/`pytest.ini` (`pytest`); `go.mod` (`go test ./...`, `go build ./...`); `Makefile` targets `test`/`build`. Several stacks: join with ` && `, frontend first.
- `url`: only when there is a `dev` command; the port from the Vite/Next config or the dev script, else `http://localhost:5173`.
- Areas: the top-level folders under `src/` (or the source roots when there is no `src/`), at most 5, each `area.<folder>: <path>/`. Lowercase, hyphens only.
- Themes and widths: theme names from the CSS (`data-theme`, `prefers-color-scheme`); no guess means the defaults in F§1.

## 2. Round A: the repo

One AskUserQuestion call, 4 questions, guess first with ` (recommended)`:

| header | question | options |
|---|---|---|
| `Prefix` | Which ticket id prefix should this repo use? | the guess; one other plausible prefix |
| `Test` | Which command runs the tests? | the guess; "No test command" |
| `Build` | Which command builds or type-checks? | the guess; "No build command" |
| `Areas` | Which areas should area notes cover? | the guessed list, comma-separated; "No areas yet" |

Each option's description says where the guess came from (e.g. "package.json scripts.test"). No guess: offer "None for now (recommended)" and one example.

## 3. Round B: web UI (only when `url` was guessed) and how the owner works

One call, up to 4 questions, same rules. Skip a question whose answer cannot change anything.

| header | question | options (key written) |
|---|---|---|
| `Themes/widths` | Which themes and widths should Shoot cover? | the guess or defaults; "Mobile and desktop only" (`shoot.widths: 375, 1280`) |
| `Round size` | How many questions per Question Time round? | "4 (recommended)", "3", "2" (`qt.round`) |
| `Pack Up` | Should Pack Up ask when the next step is unclear? | "Ask (recommended)", "Never, just write it" (`packup.questions`) |
| `Skills off` | Which skills should stay off? | "None (recommended)", "Coderot", "Reel", "Coderot and Reel" (`skills.off`) |

No `url`: leave out themes and widths. Defaults are not written: only a key whose answer differs from F§1's default goes in the file.

If no answers come back, give the read-back with the guesses marked `(unconfirmed)` and stop.

## 4. Read-back

Per F§4, groups **Tickets** (prefix, folder, archive, index command), **Commands** (build, test, dev, url), **Areas** (one per line), **Habits** (the preference keys that differ from their defaults, else "defaults"). A correction gets a fresh, complete read-back.

## 5. Write on yes

Create `.peep/` and write `.peep/setup.md` in the shape of `${CLAUDE_PLUGIN_ROOT}/references/examples/.peep/setup.md`: only keys with a value, `tickets` on its own line with a trailing `/`. Body: `# Repo setup`, then `Written by /peep:walk on <YYYY-MM-DD>. Edit by hand if a command changes.`

A rerun with a setup present starts from its values, and rewrites the whole file, keeping any key the rounds did not cover and the body notes.

Do not commit or stage it; the owner decides. Say `.peep/setup.md new, uncommitted` in the reply (or `updated, uncommitted`).
