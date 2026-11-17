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
- [ ] Widget installer download from GitHub Releases, which the offer links to
- [x] No publish without the owner's explicit yes

**Done when:** installing the pack from a local marketplace and running a skill shows the offer once, and the link points at the release asset.

## Checks

1. pass: the root `.claude-plugin/marketplace.json` lists `peep` from `./plugin`. `claude plugin validate .` and `claude plugin validate plugin` both pass, so the widget's bundled marketplace still validates.
2. pass: the pitch line "it tells you what to do next" is in the `marketplace.json` and `plugin/.claude-plugin/plugin.json` descriptions.
3. pass: the first run with no widget printed the offer line in the final reply and wrote `widget.offer: 2026-11-17` to `.peep/runs`. Evidence: a headless child `claude -p "/peep:tweet" --plugin-dir plugin` in a throwaway repo with an empty `APPDATA` (repo4). A first attempt wrote the key but dropped the line from the reply, so the line moved into the preamble's Last step, just before the Why line.
4. pass: the second run in the same repo shows no offer (repo4, run 2).
5. pass: with `$APPDATA/PeepThis/endpoint.json` present there is no offer and no key (repo5).
6. owed to the owner: the link `https://github.com/benbullock03-ship-it/peep-this/releases/latest/download/PeepThis-setup.exe` resolves only once the GitHub repo exists and a release carries that asset. This repo has no git remote yet.

## Owed before done

- Owner: create the GitHub repo `benbullock03-ship-it/peep-this`, push, and publish a release whose installer asset is renamed to `PeepThis-setup.exe` (Tauri names it `Peep This_<version>_x64-setup.exe`). A publish needs the owner's explicit yes.
- Owner, only for a later plugin directory submission: `claude plugin validate plugin --profile directory` fails on no LICENSE (the licence choice is the owner's) and warns on no icon (art comes from the owner).

## Decisions taken without a ruling

- The GitHub repo is named `peep-this`, after the marketplace name. If the owner picks another name, the URL in `plugin/references/preamble.md` and `plugin/README.md` changes.
- The offer links to `releases/latest/download/PeepThis-setup.exe`, a fixed asset name, so the link never goes stale between versions.
- Widget detection is `$APPDATA/PeepThis/endpoint.json`, which the widget writes on every start. Checking settings.json for the hooks was not used: it cannot be faked in a test without moving HOME.
- The marketplace is listed twice under the one name `peep-this`. The repo root copy is for `marketplace add <owner>/peep-this`; `plugin/.claude-plugin/marketplace.json` stays for the widget installer's bundled copy (`install.rs`). Whichever is added first wins, and both serve the same plugin.
- The offer is remembered per repo in `.peep/runs`, as the ticket says, so each new repo gets the offer once.
