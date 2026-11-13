---
id: AV-07
title: Statusline wrapper, context bars and rate-limit header
status: done
depends: [AV-05, AV-04]
attended: false
---

# AV-07 — Statusline wrapper, context bars and rate-limit header

**What to build:** `peepthis-statusline.exe` per spec §6.3, and the UI that shows its data: per-row context bar and the header's 5h/7d rate limits (spec §3.2).

- [x] Wrapper runs the recorded original command with identical stdin and prints its stdout byte-for-byte
- [x] POST is fire-and-forget; overhead measured and recorded here (budget under 50ms)
- [x] Works when the original is a `sh` script (the owner's current statusline is `statusline-command.sh` run by Git Bash)
- [x] Context % = 100 − `remaining_percentage`; rate limits from the latest payload of any session, age on hover
- [x] Header and bars hidden until data arrives

**Done when:** with the wrapper configured by hand, the owner's statusline looks unchanged and the widget shows context and rate limits.

## Notes

- Binary: `src-tauri/src/bin/peepthis-statusline.rs` (std + serde_json only, 397 KB release). `Cargo.toml` gains `default-run = "peep-this"` so `cargo run` / `tauri dev` still pick the widget.
- Original-file format lives in `src-tauri/src/statusline_original.rs`, exported from the lib for AV-08 and pulled into the bin with `#[path]` so the bin never links Tauri.
- UI: `src/limits.ts` (header meters + helpers, tests in `src/limits.test.ts`), wired in `src/main.ts`; `?demo` shows 5h 42% · 7d 18%. The row context bar from AV-04 already fills from `contextPct`.

### Overhead (release build, Git Bash, 30 runs per mean, run from a bash loop)

Original = `sh tiny.sh` (reads stdin, prints `ok`) so the script's own variance (~480–650ms for the owner's real script) does not drown the number.

| Case | Direct | Wrapper | Overhead |
|---|---|---|---|
| Widget up (python stand-in listener answering 204; 90/90 POSTs received) | 59–72ms | 86–95ms | ~20–30ms |
| Widget down (endpoint.json points at a closed port) | 58–67ms | 85–95ms | ~25ms |
| No endpoint.json | 57–62ms | 85–90ms | ~28ms |
| Original needing a shell (`… \| cat`) | 58ms | 119ms | ~60ms (extra bash layer) |

The owner's real statusline (`sh ~/.claude/statusline-command.sh`) produced byte-identical output (91 bytes, ANSI codes included) through the wrapper, and the wrapper's overhead was lost in that script's own run-to-run noise. Method: `PEEPTHIS_DIR` pointed at a throwaway temp dir with its own `statusline-original.json` and `endpoint.json`; nothing under `%APPDATA%\PeepThis` was touched.

## Decisions taken without a ruling

- **Original-file format:** `{"statusLine": <original settings.json value, verbatim>}`, so uninstall can restore `type`, `padding` and any unknown fields exactly; the wrapper reads only `statusLine.command`. A missing file, bad JSON or empty command prints nothing and exits 0. A UTF-8 BOM is tolerated.
- **How the original runs:** Claude Code runs statusLine commands through Git Bash on Windows (`~` in the owner's command only works under a POSIX shell). The wrapper first tries a **direct exec** when the command has no shell syntax (no quotes, `$`, pipes, globs, backslashes, `=` and so on): split on whitespace, expand a leading `~`/`~/` from `HOME` then `USERPROFILE`, resolve the bare program name to `<name>.exe` on PATH. This skips one bash start-up (~45ms) and is what keeps the owner's command under budget. Anything else goes through a shell: `CLAUDE_CODE_GIT_BASH_PATH`, else `bash.exe`/`sh.exe` on PATH, else Git for Windows' `bin\bash.exe` found from `git.exe` on PATH, else `cmd /C`. PATH entries under `Windows\System32` and `Microsoft\WindowsApps` are skipped (WSL `bash.exe` stubs).
- **Output and exit code:** the child inherits the wrapper's stdout and stderr handles, so output is byte-for-byte with no copying; the exit code passes through. Children get `CREATE_NO_WINDOW`. stdin goes in from a writer thread so a child that ignores stdin can't deadlock.
- **Fire-and-forget POST:** a thread, not a detached process. It starts the POST as soon as stdin is read, in parallel with the original (raw HTTP/1.1 over `TcpStream`, 200ms connect timeout, reads the status line). On exit the wrapper waits for it at most until 25ms after its own start, and a slow original already covers that. A refused loopback connect on Windows retries for about 2s, and the deadline keeps that out of the statusline. Spawning a detached copy would have cost another process start (~20ms) for no gain. The spec says the POST happens "after printing", but the order doesn't matter to either side.
- **`PEEPTHIS_DIR` env override** for the wrapper's data dir (tests and measurement only). The widget's server still writes to `%APPDATA%\PeepThis` only.
- **Header meters:** `5h 42% · 7d 18%` in IBM Plex Mono 11px, label `--text-3`, value `--text-2`, with a 2px bar under each in the same neutral grey as the row context bar. No warning colour for high usage, because the spec doesn't define one.
- **Age on hover:** the title reads "updated 12s ago" (s/m/h/d). `store.set_rate_limits` emits only when the percentages change, so the pushed `at` can be stale. On hover the frontend calls `get_snapshot` again for the latest `at`, which avoids touching `store.rs`.

## Done 2026-11-13

Owner confirmed the statusline is unchanged with the integration installed (AV-08/AV-17 now configure the wrapper), and the installed widget showed a live row's context % and the header's 5h 3% / 7d 9% (screenshot).

The check that was owed:

Owner's live check with the wrapper configured by hand (AV-08 will automate this):

1. Build and copy the wrapper somewhere a rebuild won't lock (PowerShell, repo root):
   ```powershell
   cd src-tauri; cargo build --release --bin peepthis-statusline; cd ..
   New-Item -ItemType Directory -Force "$env:APPDATA\PeepThis\bin"
   Copy-Item src-tauri\target\release\peepthis-statusline.exe "$env:APPDATA\PeepThis\bin\"
   ```
2. Record the original command:
   ```powershell
   '{"statusLine":{"type":"command","command":"sh ~/.claude/statusline-command.sh"}}' | Set-Content -Encoding ascii "$env:APPDATA\PeepThis\statusline-original.json"
   ```
3. Back up `~/.claude/settings.json`, then set `statusLine.command` to the wrapper using forward slashes, because Git Bash eats backslashes. For example: `"command": "C:/Users/<you>/AppData/Roaming/PeepThis/bin/peepthis-statusline.exe"`.
4. `npm run tauri dev`, then work in a Claude Code session. Confirm:
   - The statusline looks exactly as before, with no visible delay.
   - The row's context bar fills (hover shows "Context N% used").
   - The header shows `5h N% · 7d N%` with bars, and hovering shows "updated Ns ago".
5. To revert, set `statusLine.command` back to `sh ~/.claude/statusline-command.sh` (or restore the backup).

Open for AV-08: how the bundled app ships `peepthis-statusline.exe` (a Tauri `externalBin` sidecar or a copy next to the main exe), and the path the installer writes into settings.json.
