---
id: AV-135
title: Version gate tray line
status: doing
depends: [AV-134]
---

# AV-135: Version gate tray line

**What to build:** when a session reports a Claude Code major or minor version that has no fixture folder, show one quiet line in the tray: "untested Claude Code version". Never an error (AV-131 answer 7).

- [x] Build time: list the fixture version folders (major.minor) into the backend
- [x] Runtime: compare each session's `version`. If a version is unseen, the tray menu shows the line once, naming the version.
- [x] No row state change, toast or error
- [x] A test for the major.minor comparison

**Done when:** a session file with a faked unseen version shows the tray line, and a known version shows nothing.

## Checks

1. pass: `build.rs` writes the `N.N.N` fixture folders into `PEEP_FIXTURE_VERSIONS`, and `legacy/` is left out (`tray::tests::fixture_versions_include_captured_folders`).
2. pass: the major.minor comparison works. `2.2.0` and `3.1.1` count as untested, `2.1.400` as known, and an empty or junk version gives no line (`tray::tests::untested_compares_major_minor`).
3. pass: the parser reads `version` from the 2.1.325 fixture and gives empty when the field is missing (`sessions::tests::reads_the_claude_code_version`; `cargo test`: 439 passed).
4. owed to the owner: in `npm run tauri dev`, a session file with a faked `"version": "9.9.0"` shows a disabled "untested Claude Code version 9.9" line at the top of the tray menu, and a 2.1.x session shows nothing.

## Owed before done

- Owner check 4 in `npm run tauri dev`: the tray menu line shows for an unseen version and stays away for a known one.

## Decisions taken without a ruling

- The check runs in the source sink in `lib.rs`, before `store.ingest`, so the row, state and snapshot are untouched. `SessionInfo` gains `version`, and the frontend never sees it.
- The line is a disabled menu item with a separator at the top of the tray menu. It names the first untested major.minor seen and stays until restart. A second untested version adds no second line.
