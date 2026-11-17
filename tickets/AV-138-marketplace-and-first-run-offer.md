---
id: AV-138
title: Marketplace listing and first-run widget offer
status: doing
depends: []
---

# AV-138: Marketplace listing and first-run widget offer

**What to build:** one install path, matching ADR 0001. The `peep` pack is listed in the Claude Code plugin marketplace, and its first run offers the widget download (AV-131 answer 11).

- [x] Marketplace manifest and listing text for `plugin/`, using the pitch line
- [x] First Peep skill run with no widget detected: one line offering the widget download link. Asked once and remembered in `.peep/runs`.
- [x] Widget installer download from GitHub Releases, which the offer links to (draft; Publish owed)
- [x] No publish without the owner's explicit yes

**Done when:** installing the pack from a local marketplace and running a skill shows the offer once, and the link points at the release asset.

## Checks

1. pass: the root `.claude-plugin/marketplace.json` lists `peep` from `./plugin`. `claude plugin validate .` and `claude plugin validate plugin` both pass, so the widget's bundled marketplace still validates.
2. pass: the pitch line "it tells you what to do next" is in the `marketplace.json` and `plugin/.claude-plugin/plugin.json` descriptions.
3. pass: the first run with no widget printed the offer line in the final reply and wrote `widget.offer: 2026-11-17` to `.peep/runs`. Evidence: a headless child `claude -p "/peep:tweet" --plugin-dir plugin` in a throwaway repo with an empty `APPDATA` (repo4). A first attempt wrote the key but dropped the line from the reply, so the line moved into the preamble's Last step, just before the Why line.
4. pass: the second run in the same repo shows no offer (repo4, run 2).
5. pass: with `$APPDATA/PeepThis/endpoint.json` present there is no offer and no key (repo5).
6. owed to the owner: the link `https://github.com/benbullock03-ship-it/peep-this/releases/latest/download/PeepThis-setup.exe` resolves only once the GitHub repo exists and a release carries that asset. This repo has no git remote yet.

7. (added) pass: the public repo `github.com/benbullock03-ship-it/peep-this` exists and main is pushed. Raw `.claude-plugin/marketplace.json` on main returns 200.
8. (added) pass: the draft release v0.1.0 (not a prerelease) carries `PeepThis-setup.exe`, a fresh `npm run tauri build` from 68c96fa. The latest/download link returns 404 while it is a draft.
9. (added) pass: `claude plugin validate plugin --profile directory` passes with the MIT LICENSE; the only warning left is no icon.

## Owed before done

- Owner: click Publish on the v0.1.0 draft (github.com/benbullock03-ship-it/peep-this/releases). Until then the offer link 404s.
- Owner, before a plugin directory submission: a square PNG icon at `plugin/.claude-plugin/icon.png`, 512 to 2048 px (the art is the owner's).

## Owner rulings 2026-11-17

- Public repo `peep-this`; draft release v0.1.0; MIT licence; first push is committed main only.
- Before the public push, author and committer emails in all 339 commits were rewritten to the GitHub noreply address, in a clone. Trees are identical. Local main was moved to the rewritten history, and the old hashes are on `backup/pre-noreply-rewrite`. Repo `user.email` is now the noreply address.

## Decisions taken without a ruling

- The GitHub repo is named `peep-this`, after the marketplace name. If the owner picks another name, the URL in `plugin/references/preamble.md` and `plugin/README.md` changes.
- The offer links to `releases/latest/download/PeepThis-setup.exe`, a fixed asset name, so the link never goes stale between versions.
- Widget detection is `$APPDATA/PeepThis/endpoint.json`, which the widget writes on every start. Checking settings.json for the hooks was not used: it cannot be faked in a test without moving HOME.
- The marketplace is listed twice under the one name `peep-this`. The repo root copy is for `marketplace add <owner>/peep-this`; `plugin/.claude-plugin/marketplace.json` stays for the widget installer's bundled copy (`install.rs`). Whichever is added first wins, and both serve the same plugin.
- The release is not marked prerelease, because `releases/latest` skips prereleases and the link would never resolve.
- The offer is remembered per repo in `.peep/runs`, as the ticket says, so each new repo gets the offer once.
