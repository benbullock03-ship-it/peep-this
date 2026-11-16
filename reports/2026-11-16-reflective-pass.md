# Reflective pass: research findings (2026-11-16)

Inputs: all 15 skills in `plugin/skills/`, `CONTEXT.md`, open tickets, Matt Pocock's skills (local cache 1.2.0 + github main), competitor repos, monetisation and teaching research. No `.peep/reel.md` in this repo yet, so no north star to check against.

## 1. Matt Pocock: what to take

- **improve-codebase-architecture** (user-invoked only): scope by YAGNI (named direction, else `git log` hot spots, ~20 commits); a fresh-context subagent hunts friction (shallow modules, concepts smeared over many files, leaks across seams, untested code); **deletion test** as the only filter ("would deleting it concentrate complexity?"). HTML report in tmpdir: legend, one card per candidate (strength badge Strong / Worth exploring / Speculative, files, before/after diagram, one-line problem + solution, wins of 6 words or fewer, ADR conflict callout), one Top recommendation, then "which would you like to explore?" and a grilling loop. A rejected candidate gets an ADR so it is not re-suggested.
- Principles: facts are looked up, decisions are asked; small composable skills, not a process that owns you; "rate of feedback is your speed limit"; a garbage codebase gets garbage output; ~120k-token smart zone; clear per ticket.
- Take: YAGNI scoping, deletion test, badge + top-pick + stop-and-ask report, ADR on rejection, `disable-model-invocation` on heavy skills, one router skill.
- Leave: issue-tracker machinery (triage, wayfinder), banned-word policing, Tailwind/Mermaid CDN (breaks offline rule), one-question-at-a-time grilling.

## 2. Competitors

- Closest: **vibe-notch** (ex Claude Island, 2.5k stars, macOS notch, hooks, approve/deny). All notch tools are macOS-only; no Windows tray/pet app over ~50 stars. **Windows is open ground.**
- Most-praised cheap features Peep lacks: focus-aware toast suppression (spec only suppresses when the widget is focused), sound per attention state (off by default), rate-limit reset countdown + "will you hit it" projection (ccusage, Usage-Monitor), git branch / `/rename` title on row and toast, last assistant line preview, "only needs-me" filter, user override of hashed colour/accessory (`/buddy` reroll repos show people want to own their creature), drift mood (claude-code-tamagotchi).
- Traps: worktree orchestration / launching sessions, cost dashboards (accuracy fights), full transcript viewer or chat input.
- Risks seen: PostToolUse racing after Stop with background Bash (vibe-notch); one bad hook key voids settings.json; default-on telemetry backlash. State "offline, no telemetry" loudly.
- Discovery: awesome-claude-code (58k stars) listing.

## 3. Money

- Realistic scale: komorebi (Windows tool, 10k stars, 106k downloads) made $1,862 from GitHub Sponsors in 2024. Bongo Cat at 170k daily users netted $2-4k/month and drew bots and speculation.
- Fees: GitHub Sponsors 0% (personal); Ko-fi 0% tips; merchant of record (Lemon Squeezy, Paddle, Polar) ~5% + $0.50, handles EU VAT. UK seller of digital goods owes EU VAT from the first sale without one. Price floor £4-5.
- Fit, best first: Sponsors/Ko-fi link + supporter accessory; pay-what-you-want cosmetic pack via a merchant of record; earned cosmetics for rhythm milestones (no revenue, but teaches and retains); artist packs with credit/split; merch. Steam later or never (audience mismatch).
- Never: random or tradeable drops, paywalling identity/states/skills, upsell nags.
- Constraint: new accessories are art, and art comes from the owner (`design/peep.js`).

## 4. Teaching

- Facts with numbers: Chroma context rot (focused ~300 tokens beat ~113k on every model); attention is n² budget (Anthropic); lost in the middle (arXiv 2307.03172); dumb zone ~40% (Horthy, on ~200k windows) or 120-150k tokens (Pocock); 2 failed corrections then `/clear` (Anthropic best practices); `/clear` free, `/compact` costs; cache ~1h subscription / 5min API key; CLAUDE.md taxes every turn; subagents return 1-2k tokens; compaction keeps root CLAUDE.md + 5 recent files.
- **Caveat:** 1M windows make % thresholds mislead (40% = 400k). Tips should key on absolute tokens (~100-150k) as well as %.
- Teach on signal, once, dismissible, with a Jump; "learned" list to revisit; never during `working`/`thinking`; no install tour; no streak guilt (count clean sessions, not misses).
- Existing hook: every skill already prints a `Why:` line for its first 3 runs. That is the right size.

## 5. Owner rulings (2026-11-16)

- Ducks asks "how deep? 1-5" only when nothing is loose. Ladder by cost: 1 today's Ducks; 2 + git-log hot spots (~20 commits); 3 + deletion-test candidate cards; 4 + fresh-context subagent sweep and goose map; 5 + HTML report. Each level states rough cost first.
- Level 5 extends the goose page (offline, peeps inlined): cards with badge, before/after mini-diagram, one-line problem/fix, ELI5 fold, one top pick, then stop and ask. Basic import map by default; graphify used if built, offered once.
- Rejected candidate: one line in the area note; ADR only when hard to reverse or surprising.
- Tweet: plugin PreToolUse hook on Edit/Write nudges a tweet when another live session edited the file in the last 30 min. Two failures of the same tool nudge a tweet only with a live peer in the repo, else `/clear` with what was learned.
- Trim: Unpack folds into Ducks, `hello` goes (15 to 13).
- Model-invocable only: ticket, packup, tweet, shoot. Rest `disable-model-invocation`, descriptions one line.
- Progressive unlock in tour and floppies (skills stay callable): Big Peep after 5 ticket runs; Coderot after first ticket with tests; Goose after 3 tickets; Sneak and Scrub after first UI ticket; Shoot core when setup has a url.
- Coderot with no argument targets the hot-spot area. QT: 4 per round for first 3 runs, then whole frontier as a numbered list.
- No reel: one Ducks Watch line, at most daily.
- Teaching: signal tips (context past ~100-150k tokens, 2 failed retries, cache idle >1h, first subagent, auto-compact), once each, never while working/thinking, with a Jump; context line tip carries a Pack Up Jump. Learned list on a tray page, grouped by topic, ELI5 fold and source per entry. Skills keep first-3-runs Why lines, which also feed the list. ELI5 only in report cards and learned list.
- Money: Sponsors/Ko-fi link in tray About + README; honour-code supporter toggle; earned cosmetics for 5 rhythm milestones (first Pack Up, first 0-loose Ducks, 10 tickets closed, first 0-survivor Coderot, first architecture candidate acted on). Owner draws 6 accessories. Free set and hash never locked.
- Widget features: focus-aware toasts; branch + `/rename` title on row and toast; own-your-peep override in widget settings keyed by cwd hash; sound on waiting/error, off by default, never for subagents.
- Growth: README states offline, no telemetry, Windows-first; awesome-claude-code submission after trims land (outward step, needs a yes at the time).

## 6. Observations from the skills themselves

- 15 skills. Overlaps: Ducks step 1 already reads the Pack Up note, Unpack re-reads it and checks drift; three UI skills (sneak, scrub, shoot); `hello` is install-check only.
- Ducks is deliberately the cheap present-tense check (one test run, no writes before yes). A 5-level architecture review inside it changes its cost profile.
- Goose already builds an offline graph page with peeps on areas: the natural canvas for an architecture report.
- Tweet's description already lists auto triggers, but a model rarely remembers to tweet mid-task. A deterministic nudge needs a hook (PreToolUse on Edit/Write against other live sessions' recent edits, which goose's map script and the clash mood already compute).
- Run counts: `goose`, `walk` never counted here though goose has run; `.peep/setup.md` was written by Ducks, not Walk.
