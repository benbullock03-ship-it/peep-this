---
id: AV-22
title: Risk tier classifier for permission requests
status: todo
depends: []
attended: false
---

# AV-22 — Risk tier classifier for permission requests

**What to build:** A pure function `risk::classify(tool, input, cwd) -> Tier` (spec §11.2) that every approval surface uses to pick its guard. No I/O.

| Tier | What lands here | Guard |
|---|---|---|
| `safe` | Read-only tools; `Edit`/`Write`/`NotebookEdit` on a path inside the session cwd | One click, toast allowed |
| `shell` | `Bash`/`PowerShell` not matched below; edits outside cwd; MCP and unknown tools | One click, full command shown, no toast approve |
| `destructive` | Recursive or forced delete, `git push --force`, `git reset --hard`, `git clean`, publish/release/deploy commands, pipe-to-shell (`curl … \| sh`, `iwr … \| iex`), SQL `DROP`/`TRUNCATE`, disk/registry tools, writes to settings, hook, credential or `.env` files | Hold 1s to confirm, never from a toast, never auto-allowed |

- [ ] Fails closed: anything unparsed or unknown is `shell`, never `safe`
- [ ] Compound commands (`&&`, `;`, `|`, subshells, `bash -c`, `powershell -Command`) take the highest tier of any part
- [ ] Returns the matched span so the card can highlight the risky token
- [ ] Both shells covered: POSIX and PowerShell spellings (`rm -rf`, `Remove-Item -Recurse -Force`, `del /s`)
- [ ] Path check normalises separators and case, and resolves `..` before the inside-cwd test
- [ ] Table-driven tests: at least 60 real commands, including evasions (quoting, aliases, env prefixes)

**Done when:** the test table passes and the owner has read the destructive pattern list and added what is missing.

Decide: is plain `git push` destructive? Recommended: `shell` (reversible, and the user's own rules already gate it by asking).
