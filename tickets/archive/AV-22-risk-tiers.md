---
id: AV-22
title: Risk tier classifier for permission requests
status: done
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

## Built

`src-tauri/src/risk.rs`: `classify(tool, input, cwd) -> Verdict { tier, field, span, reason }`. `span` is a byte range into `input[field]` (`command`, `file_path` or `notebook_path`). Hand-written lexers for POSIX, PowerShell and cmd. No new crates. Tests: 146 command rows, 24 edit rows, span, tool and nesting tests (`cargo test risk`).

Destructive patterns, for the owner to review:

- **Delete:** `rm`/`del`/`erase`/`rd`/`rmdir`/`ri`/`Remove-Item` with recursive or force (`-rf` clusters, `--recursive`, `--force`, PowerShell prefixes `-r`/`-Rec`/`-fo`/`-Recurse:$true`, cmd `/s` `/q` `/f`); `rimraf`, `shred`, `srm`, `wipe`; `find -delete` and `find -exec <destructive>`; `rsync --delete*`.
- **Git:** push with `--force*`, `-f`, `--mirror`, `--delete`/`-d`, `--prune`, a `+ref` or `:ref`; `reset --hard`; `clean` (except `-n`); `checkout`/`switch -f` or `.`; `restore .` (unless `--staged`); `branch -D`; `stash drop|clear`; `rm -r|-f` (unless `--cached`); `filter-branch`/`filter-repo`; `reflog expire|delete`; `update-ref -d`.
- **Publish and deploy:** any of the first three subcommands `publish`, `unpublish`, `deploy`, `*-deploy`, `undeploy`, `yank`, `deprecate`, `release` (not before list/view/show; not as a flag's value); runner scripts (npm, pnpm, yarn, bun, make, gradle, mvn and others) whose name contains deploy or publish, or starts with release; `push` for docker, podman, helm, gem, nuget, `dotnet nuget`; terraform/tofu/pulumi/kubectl/cdk `apply|destroy`, `pulumi up`, `kubectl replace|drain`, helm `install|upgrade|uninstall|rollback`; delete/destroy/rm/terminate subcommands on cloud CLIs (aws, az, gcloud, gh, kubectl, vercel, netlify, firebase and others); `vercel` with no subcommand or `--prod`; `twine upload`; semantic-release, release-it, goreleaser, np; `docker volume rm`, `docker … prune`.
- **Pipe to shell:** any pipe into sh/bash/zsh/fish, python/node/perl/ruby/php, `cmd`, `powershell`/`pwsh`, `source` when it reads stdin; `iex`/`Invoke-Expression` when piped or given a dynamic argument; any shell or interpreter given a process substitution or command substitution that contains curl/wget/iwr/irm/WebClient; `powershell -EncodedCommand`.
- **SQL:** `DROP TABLE|DATABASE|SCHEMA|INDEX|VIEW|USER|ROLE|…`, `TRUNCATE TABLE` (or a leading TRUNCATE), `DELETE FROM` with no WHERE, `dropDatabase`, `FLUSHALL`/`FLUSHDB`; `dropdb`, `dropuser`, `prisma migrate reset`, `--force-reset`, `--accept-data-loss`, `db:drop|db:reset|db:purge`. Not checked inside the arguments of git, echo or grep-family commands.
- **Disk and registry:** format, Format-Volume, diskpart, fdisk/sfdisk/gdisk, parted, wipefs, mkfs*, mke2fs, mkswap, Clear-Disk, Initialize-Disk, Remove-Partition, bcdedit, `dd of=`, `vssadmin delete|resize`, `wmic shadowcopy delete`, `cipher /w`, `diskutil erase*`; `reg add|delete|import|restore|load|unload|copy`, `regedit`/`regini` with arguments, and PowerShell Set-/New-/Remove-/Rename-/Move-/Copy-/Clear- cmdlets (and their aliases) on `HKLM:`/`HKCU:`/`Registry::` paths.
- **Protected files** (an edit, a redirect, tee, `-o`/`-OutFile`, cp/mv destination, `sed -i`, Set-Content/Out-File, rm): `.env`, `.env.*`, `.envrc`; `.claude/settings*.json`, `.claude/hooks/`, `.claude.json`, `managed-settings.json`; `.git/hooks/`, `.git/config`, `.husky/`; `.ssh/`, `.aws/`, `.gnupg/`, `.kube/`, `.azure/`, `gcloud/`, `.docker/config.json`, `.npmrc`, `.pypirc`, `.netrc`, `.git-credentials`, `.pgpass`, `credentials(.json)`, `.htpasswd`, `id_rsa*`/`id_ed25519*`/`id_ecdsa*`, `*.pem|key|pfx|p12|jks|keystore|ppk`; shell startup files (`.bashrc`, `.zshrc`, `.profile`, PowerShell profiles).
- **Inline code:** `python -c`/`node -e` and similar containing rmtree, os.remove, unlink, rmSync, rimraf, Remove-Item or `rm -rf`.
- Nesting deeper than 6 levels: a command too deeply nested to inspect counts as destructive.
- **Power:** shutdown, reboot, halt, poweroff, logoff, Stop-Computer, Restart-Computer (not `shutdown /a` or `-c`); `systemctl poweroff|reboot|halt|kexec`; `init`/`telinit 0|6`.
- **Permissions:** chmod/chown/chgrp/setfacl with `-R`/`--recursive`; `icacls … /t` with grant/deny/remove/reset/setowner/inheritance; `takeown /r`.
- **Processes:** pkill, killall; `kill … -1` or `kill … 0`; `taskkill /im|/fi`; Stop-Process/spps/PowerShell `kill` with `-Name`, or fed by a pipe. Kill by PID stays `shell`.
- **Git exec settings:** `git config` setting `core.hooksPath`, `core.fsmonitor` or `core.sshCommand` (reading or `--unset` is `shell`); `git -c` with one of those keys.
- **Deploy scripts by file name:** a command or a script argument to a shell, interpreter or PowerShell whose file name contains deploy or publish (`node scripts/deploy.js`, `./deploy.sh`, `-File .\deploy.ps1`).
- ANSI-C `$'…'` strings are decoded (`\xHH`, octal, `\u`, `\U`, `\cX`, letter escapes) before matching.

Not covered: aliases and functions defined earlier in the same command, symlinks inside the cwd that point outside it (no I/O).

## Decisions taken without a ruling

- Plain `git push` is `shell`, as recommended.
- `classify` returns a `Verdict` (tier, field, span, reason) rather than a bare `Tier`, so the card gets the span the checklist asks for.
- Every Bash/PowerShell command is at least `shell`, including read-only ones like `ls`. Only non-shell read tools are `safe`.
- WebFetch, WebSearch, TodoWrite and BashOutput count as reads (`safe`).
- `.env.example`, `.env.sample` and `.env.template` are exempt from the `.env` rule. `.envrc`, shell startup files, `.git/config` and `.husky/` count as hook files.
- Deleting a protected file (`rm .env`) counts as writing it, so it is destructive.
- `git clean -n` (dry run) is `shell`. `dotnet publish` is `shell` because it is a local build.
- Any pipe into an interpreter that reads stdin is destructive, not only pipes from curl or iwr.
- PowerShell `-EncodedCommand` and nesting deeper than 6 levels are destructive, because the classifier cannot see what runs. This goes beyond "unparsed is shell" on purpose: hiding the command is treated as an evasion.
- A dynamic command name (`$x -rf`, `& ('r'+'m') -r`) is checked against the delete-flag rule.
- Plain `rm file` and `Remove-Item file`, with no recursive or force flag, are `shell`.
- Gaps pass (2026-11-14): `kill <pid>` and `taskkill /pid` are `shell` (agents stop their own dev servers); by-name and all-process kills are destructive. `git config core.hooksPath .githooks` is destructive even though husky-style setups do it legitimately, since it makes every later commit run that folder. Only deploy/publish in script names count; "release" is too often a local build.
- Inside cwd: separators and case are normalised, `..` is resolved lexically, NTFS `::$DATA` suffixes and trailing dots are stripped, Git Bash `/f/…` maps to `f:/…`, and `~`, `$VAR` or drive-relative `c:foo` paths count as outside.

## Done 2026-11-14

Owner read the destructive pattern list under `## Built` and had nothing to add. The test table passes (`cargo test risk`), with the gaps pass adding power, recursive permissions, process kills, git exec settings, deploy scripts and ANSI-C strings.
