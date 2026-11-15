---
id: AV-32
title: While-you-were-away digest
status: in-progress
depends: [AV-26]
attended: false
---

# AV-32 — While-you-were-away digest

**What to build:** Stub. On return from being away, one card that says what happened, ordered by what needs the user first (spec §11.9). Built from events only; no model call.

- [ ] Away: screen locked, or no input for 5 min (setting). Return: unlock or first input
- [x] Card groups sessions: blocked on you, failed, finished, still running. Per session: time taken, files changed (count and names), commands run (count), last assistant line, approvals released to the terminal while away
- [ ] Each entry focuses its session; blocked entries open the inbox
- [ ] No card when nothing changed. Dismissed by Esc or any decision
- [x] The per-session activity log behind it is in memory, capped, and holds paths and counts only
- [x] Tests: grouping, ordering, empty case

**Done when:** the owner locked the screen during real work, came back, and the card matched what had happened.

## Built 2026-11-15

- `src-tauri/src/away.rs`: `step` (pure presence machine: Here / Away{since, locked}), `Activity` (capped per-session log of `Note`s), `digest` (pure: entries grouped and ordered), `Away` (poll thread, card state, `digest` event), Windows probe (`GetLastInputInfo` + `GetTickCount`; `OpenInputDesktop` returns null on the lock screen), commands `get_digest`, `dismiss_digest`, `away_after`, `set_away_after`. `Cargo.toml` gained the `Win32_System_StationsAndDesktops` and `Win32_System_SystemInformation` features of the existing `windows-sys` (no new crate).
- `store.rs`: the log lives in the store (`note`, `digest`), pruned on each tick; never in the snapshot.
- `ingest/server.rs`: `activity_note` turns a hook payload into a note (JSON parsed once now, `parse_value`); `lib.rs`: the trail recorder notes releases and dismisses the card on any widget allow/deny.
- Frontend: `src/digest.ts` (pure helpers), `src/digestCard.ts` (card at the top of the panel list, Esc and × dismiss), `main.ts` wiring, `settings.ts` "Away after" select, `styles.css`, `?demo&digest` sample in `demo.ts`.
- Tests: `away.rs` 15 (presence: idle, lock, idle-then-lock; cap and partial counts; clipped line and paths; empty case; group order blocked, failed, finished, running; ended/removed sessions and a failed shell; counts and the open turn's time; running time and file list cap; state change alone; prune; setting round trip; return emits once and dismiss clears; startup scan is not news), `server.rs` 2 (notes from fixtures, hooks feed the digest), `src/digest.test.ts` 5.

## Decisions taken without a ruling

- Away starts at the last input before it was detected (now − idle), so the five idle minutes are in the window. A lock is away at once; input on the lock screen does not end it, the unlock does. The window never opens earlier than 10s after the widget started, so sessions found by the startup scan are not news.
- Threshold choices 1, 2, 5, 10, 15, 30 min (`awayAfterMin` in `%APPDATA%\PeepThis\settings.json`, read-modify-write like the other keys). `PEEPTHIS_AWAY_SECS` overrides it in seconds, for dev and live checks only.
- On the card: any session with a note or a state change during the absence. Groups from the state at return: `waiting` or a held request = blocked; `error`, `stuck`, or a background shell that failed during the absence = failed; `thinking`/`working` = running; anything else, and ended or already removed sessions, = finished.
- Inside a group: blocked by longest waiting first, others by latest activity first.
- Time taken: from the prompt of the turn that was running when the user left (else the first activity in the window) to the last activity for finished sessions, or to the return for the rest.
- Files: from successful `PostToolUse` of `Edit`, `Write`, `MultiEdit`, `NotebookEdit` only (a denied or failed edit changed nothing); relative to the cwd as the rows show them; 8 names listed by base name, full paths in the hover, the rest counted. Commands: every `Bash`/`PowerShell` `PostToolUse` or `PostToolUseFailure` (a failed command still ran), subagents included.
- Last assistant line: the first non-blank line of `Stop.last_assistant_message`, 160 characters. In memory only, like the rest of the log; the hook fixtures' leak test still holds for `Routed`.
- Log cap 256 notes per session; once notes inside the window were dropped, counts show as `N+`. Logs of sessions gone from the store are kept a day.
- Strip mode on return: nothing changes mode (auto-expand is in §12.9's not-taken list). The card waits at the top of the panel and shows on the next peek, toggle or click; no card in the strip or the flyout.
- Esc: the surface grill's "No Esc" is about mode toggling. Esc dismisses this card only, while the widget has focus, and not while the keyboard inbox or settings own it. A × in the card head also dismisses (mouse users rarely have the widget focused).
- "Any decision" = an allow or deny made in the widget (card, inbox, toast); releases and rule auto-allows do not dismiss. Clicking an entry does not dismiss.
- Click: a blocked entry with a held approval opens the inbox (oldest first, as the inbox does); blocked on a question, failed, finished or running entries focus the terminal; ended sessions are not clickable.
- A new absence keeps an undismissed card until the next return builds a new one, which replaces it; a return with nothing new leaves it.

## Owed before done

Live check by Claude in `npm run tauri dev` with `PEEPTHIS_AWAY_SECS=20`, no screen lock (owner away), synthetic hook POSTs for three fake sessions (`0a320a32-aaaa/bbbb/cccc-…`, cwd `C:/peeptest/av32`) and a `mouse_event` nudge as the return. Screenshots in `C:\Users\benbu\AppData\Local\Temp\claude\30a9097ea8fcfb1f\4987ae0e-47a2-463f-9c4f-ed2e98c06b1a\scratchpad\AV-32\`.

- Seen: idle detection (owner idle, "away since" in the log at startup); return on input ("back after 307s, digest 5 entries"). The card (`01-return-strip`, `02-card-crop`) matched what was sent: session A blocked (still at the terminal prompt after its held request timed out) with 2 files changed (`app.ts` edited twice, counted once), 3 commands, "1 approval went to the terminal", and only the first line of the Stop message; B blocked on a question; C failed with 1 command (a `PostToolUseFailure`); two real sessions under Still running with their own files and command counts. Esc dismissed the card once the widget had focus (`05a`, `05b`). Short absences between clicks (my own turnaround passed the 20s threshold) gave fresh cards listing only the sessions that changed, so the unchanged fake sessions were left out (`03-after-close`, `04-dismissed`).
- Fixed after the first look: the facts line wrapped mid-word under the ellipsis; it now wraps (`02-card-crop` is after the fix).
- Not seen live: a real lock and unlock (`OpenInputDesktop` path); "no card when nothing changed" (this session's own hooks always counted as activity; unit tests only); the × button cleanly (each click there was also a return, so the old card was swapped for a new one); dismissal by a widget allow/deny; an entry click opening the inbox or focusing a terminal (fake sessions have no PID); the Away after setting in the settings view; light theme; the card in a peek from the strip.
- Owner: lock the screen during real work for more than a minute, unlock, and check the card against what happened (the Done-when). Then: × and Esc dismiss, an allow from the inbox dismisses, a blocked entry opens the inbox, a finished entry focuses its terminal. Look at the card in both themes, and at the Away after select in settings.
- App data was backed up and restored (`trail/2026-11.jsonl` held one released entry for the fake session and was put back; `endpoint.json`, `settings.json`, `window.json` were unchanged). Dev processes stopped.
