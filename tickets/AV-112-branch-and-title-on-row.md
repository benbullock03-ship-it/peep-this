---
id: AV-112
title: Git branch and session title on the row and toast
status: doing
depends: []
---

# AV-112 — Git branch and session title on the row and toast

**What to build:** Tell several sessions in one repo apart.

- [x] Session title: the `/rename` name or ai-title when Claude Code records one (verify where: sessions file or transcript), else today's name
- [x] Git branch of the session's cwd, read cheaply (`.git/HEAD`), refreshed on state change
- [x] Row shows the title; branch as a tag on the open row; toast title `<title> · <branch>`
- [x] Parser tests with fixtures; Shoot both themes

**Done when:** two sessions on different branches of one repo show distinct titles and branches on rows and toasts; the owner confirms.

## Finding: where Claude Code records the title (2026-11-16, v2.1.325)

- `/rename`: `sessions/<pid>.json` `name` with `nameSource: "user"` (generated names carry `nameSource: "derived"`), and a `{"type":"custom-title","customTitle":...}` line in the transcript (plus a matching `agent-name` line).
- ai-title: only in the transcript, as repeated `{"type":"ai-title","aiTitle":...}` lines; the session file's `name` stays derived.
- Built: `sessions.rs` sets `SessionInfo.title` for user names; `transcript.rs` emits `TranscriptTitle`; `Session.title` = file user name, else last `custom-title`, else last `ai-title`. Branch: `src-tauri/src/git.rs`, re-read in `store.rs` `refresh_branch` on first sight, revive, cwd/pid change or state change.

## Decisions taken without a ruling

- Title precedence: session-file user name > transcript `custom-title` > `ai-title` > session-file `name`.
- Detached HEAD shows the 7-char short sha; a non-repo cwd shows no branch tag and the toast drops ` · <branch>`.
- Branch tag sits last in the open row's tags, IBM Plex Mono, ellipsised, hover title `Git branch <name>`; no glyph.
- Title replaces the name everywhere the row view is used (rows, open head, flyout, strip labels); closed shelf, settings and trail keep the session-file name.
- Branch is not re-read on a branch switch made outside the session while its state is unchanged (ticket says "on state change").

## Owed before done

- Owner close. Live check passed 2026-11-16 in `tauri dev`: scratch repo on `feat-a` plus a worktree on `feat-b`, three throwaway sessions; snapshot titles "Greeting wording question" (feat-a, feat-b) and "Sleep 30 and tea-or-coffee question" (feat-a); waiting toasts in the notification DB read `Greeting wording question · feat-a`, `… · feat-b`, `Sleep 30 and tea-or-coffee question · feat-a`.
