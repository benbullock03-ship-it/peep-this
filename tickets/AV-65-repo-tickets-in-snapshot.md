---
id: AV-65
title: Backend reads each project's open and parked tickets
status: done
depends: [AV-56]
attended: false
---

# AV-65 — Ticket shelf data

**What to build:** For each session's repo, the snapshot carries its open and parked tickets.

- [ ] Find the tickets folder from the repo setup, else `tickets/`; watch it for changes
- [ ] Parse frontmatter only: id, title, status, reason, wake; skip archive
- [ ] Sessions in the same repo share one read
- [ ] Watches the repo's archive to notice a ticket being closed (feeds the cheer mood, AV-70)
- [ ] `cargo test` for the parser with fixtures, including a malformed file (skipped, not fatal)

**Done when:** the snapshot lists a live repo's open and parked tickets and updates when one changes.

## Decisions taken without a ruling

- Repo = nearest ancestor of the session cwd holding `.git` or `.peep/setup.md`, else the cwd; `link()` (AV-64) now uses the same root, so a session in a subfolder finds titles too.
- Shape: top-level `Snapshot.repos: Repo[]` (`root`, `sessions` = live session ids, `tickets`, optional `lastClosed {id, at}`), sorted by root; one read per repo, frontend finds its repo by session id.
- `done` tickets ride the shelf (chip "review" per FLOPPY-SPEC); `closed` ones and the archive never do, so the design's "N done · show" fold has no data yet.
- A closing is an id newly in the archive or newly `status: closed` in the tickets folder, after the repo's first read; the later archive move of the same id is not a second closing. `lastClosed` is forgotten when the repo has no live sessions.
- Archive files are listed by file name only, never parsed; `archive:` from the repo setup, else `<tickets>/archive/`.
- Watcher: one `tickets` thread rereads all live repos on any change in a watched tickets or archive folder (50ms debounce) and every 5s (new sessions, folders created later); the store only re-emits when `repos` changed.
- Malformed files (no closed frontmatter, unknown status) are skipped silently, since a 5s reread would repeat any warning.
- A ticket without `id:` takes it from its file name; without `title:` it gets "".
- TS `Snapshot.repos` is optional so `src/demo.ts` (another worker's file) still type-checks; the backend always sends it.
