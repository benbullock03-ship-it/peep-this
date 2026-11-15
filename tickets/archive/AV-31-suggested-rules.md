---
id: AV-31
title: Suggested auto-allow rules
status: done
depends: [AV-22, AV-26]
attended: false
---

# AV-31 — Suggested auto-allow rules

**What to build:** The widget offers a rule after the user approves the same shape of request three times, and applies confirmed rules itself (spec §11.8).

- [x] Shape: project cwd + tool + normalised target (command with arguments generalised, or path glob). Matching is exact on the shape; no fuzzy matching
- [x] Offer appears on the card after the third matching manual allow: "Auto-allow `npm test` in this project?" Yes / Not now / Never ask
- [x] Rules live in `%APPDATA%\PeepThis\rules.json`; the widget never writes Claude Code's own permission settings
- [x] `destructive` tier is never offered and never matched, whatever the rule says
- [x] Every auto-allow is written to the trail with its rule id and shown as a tick on the row; one click on that entry revokes the rule
- [x] Settings view lists rules with use counts and a delete button
- [x] Tests: shape normalisation, threshold, destructive exclusion, revoke

**Done when:** the owner accepted a suggested rule, saw the next matching request pass with no prompt and a tick, then revoked it from the trail.

Decide: how far to generalise arguments (recommended: keep the command and subcommand, drop trailing file arguments; show the exact pattern in the offer).

## Built 2026-11-15

- `src-tauri/src/rules.rs`: `shape_of`, `command_pattern`, `path_pattern`; `Rules` (counts and open offers in memory, rules and never-ask list in `rules.json`); commands `rules_view`, `answer_rule_offer`, `revoke_rule`; `rules` event carries the full view.
- `approve.rs`: `hold` asks `Rules::auto_rule` before holding; a match replies allow at once, records `auto-allowed` with the rule id, counts a use and moves the session on. A delivered widget allow calls `note_allow`, a deny `note_deny`. `Decision` gained `rule`, which `trail.rs` writes.
- `lib.rs`: `wire_rules`; the waiting toast is skipped when the session already left `waiting` (an auto-allow answers inside the toast's 200ms delay).
- Frontend: `src/rules.ts` (pure helpers), `src/ruleOffer.ts` (offer card under the row, same card language as approvals), `rows.ts` (offer host, auto-allowed trail entries are buttons that revoke, struck through once revoked), `settings.ts` ("Auto-allow rules" list: tool, pattern, use count, cwd, Delete), `main.ts` wiring, `styles.css`.
- Tests: `rules.rs` 12 (normalisation, paths, threshold, separate counts and deny restart, destructive never offered or matched, yes/exact match/use count, revoke, never-ask and not-now, corrupt file, unreadable path, missing file), `approve.rs` 2 (offer then auto-allow without holding then held again after revoke; a hand-written rule never passes a destructive request), `src/rules.test.ts` 6.

Safety, as built: the classifier is re-run inside `auto_rule` and in `hold` on every request; `destructive` returns before any file read. Any read or parse error on `rules.json`, or an unknown `version`, means no auto-allow, no offer, and no write (revoke and accept return the error; the user's file is left as it was). Rules are created only by `answer_rule_offer` with an offer id the backend raised after real widget allows; there is no HTTP route. Nothing writes `~/.claude/settings.json` or any Claude Code permission setting.

## Decisions taken without a ruling

- Generalisation (the ticket's recommendation, applied conservatively): the command and subcommand are always kept, every other word is kept, and only a trailing run of file arguments (from the third word on) folds into `<files>`. A file argument is a word with a `/` or `\` or a file extension, not a flag, and inside the project: relative, no `..`, no drive, no `~`. Files outside the project stay literal, so they never match a `<files>` rule. The offer shows the exact pattern and a note that `<files>` means file paths inside the project.
- Commands with any of `; & | < > \` $ ( ) { } " ' * ? % ^ ! # @`, control characters, or over 300 characters get no shape and are never offered. Conservative: quoted arguments and `npm i @scope/pkg` are never offered.
- File tools (`Edit`, `Write`, `MultiEdit`, `NotebookEdit`, any tool with `file_path`/`notebook_path`): inside the project the shape is the folder and extension, `src/*.ts` (not subfolders); a name with no extension, or a path outside the project, is the exact path. Other tools use the exact trail target (url, pattern, query); tools with no target (most MCP calls) are never offered.
- cwd in a shape is lowercased with forward slashes, so `C:\Work\Demo\` and `c:/work/demo` are one project. Paths are compared lowercased (Windows).
- Counting: only allows made in the widget (card, keyboard inbox, toast) count; answers in the terminal are invisible. Counts are per project, across sessions, and live in memory only (a restart starts them again). A widget deny of the same shape restarts its count. The offer shows on the row of the session whose allow was the third.
- Not now restarts the count (offered again after three more allows). Never ask is stored in `rules.json` under `never` and survives restarts; there is no UI to clear it yet (edit the file).
- Rules apply while the widget is hidden as well: a confirmed rule is the user's own decision, and the tick is in the trail.
- The row pulse still starts for a request a rule then answers (the hook's state event lands before the broker); the toast is suppressed. Moving the rule check ahead of the state event would touch the event server; not done.
- The tick is the trail entry on the expanded row (✓ "auto", AV-26's mark); the closed row shows nothing extra.
- Offers render only in the panel, not in the strip flyout window.
- Rule ids are `r<hex ms>`; use count and last use are written to `rules.json` on every auto-allow.

## Done 2026-11-15

Owner closed it 2026-11-15 on the build installed 19:44. What was owed, as it stood:

Live check by Claude in `npm run tauri dev` with real mouse clicks (`SetCursorPos`/`mouse_event`) and synthetic `PermissionRequest` POSTs for a fake session (`0a310a31-…`, cwd `C:\peeptest\av31`). Screenshots in `C:\Users\benbu\AppData\Local\Temp\claude\30a9097ea8fcfb1f\4987ae0e-47a2-463f-9c4f-ed2e98c06b1a\scratchpad\AV-31\`.

- Seen: three allows of `npm test` (`01`–`03`); no offer after the second (`03-req3-before`, again `19-c9`); the offer after the third (`04-offer`); Yes wrote the rule to `rules.json`; the fourth request returned allow in 0.16s with no card (`06-after-auto`) and a ✓ auto entry on the expanded row (`07-trail-tick`, hover `08-hover-revoke` names the rule); settings listed it as "used once" (`09-settings-rules`); revoking from the trail struck it through (`11b-revoke-retry`) and emptied `rules.json`; the fifth request was held again (`12-req5-prompts`). A hand-written rule for `git push --force` did not pass it: held as destructive with Hold to allow (`13-destructive-held`). Settings Delete removed a rule (`14`, `15`). A second full cycle, after a deny had restarted the count, offered again on the third allow and a single click revoked (`20-offer2`, `22-expanded2`, `23-revoke-single`).
- Seen once: the first click on the trail entry, straight after closing settings (the window had just resized), did not revoke; the next click did, and in the second cycle one click did. Owner: confirm a single click revokes.
- Not seen live: Not now and Never ask (unit tests only); an offer on a subagent's request; a corrupt `rules.json` in the running widget (unit tests only); light theme.
- Owner: accept a suggested rule on a real session, see the next matching request pass with a tick, revoke it from the trail (the Done-when). Look at the offer card and the settings list in both themes.
- App data was backed up and restored (`window.json`, `settings.json`, `endpoint.json`, `trail/2026-11.jsonl`); `rules.json` did not exist before and was removed.
