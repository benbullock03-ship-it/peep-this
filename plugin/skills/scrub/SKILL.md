---
name: scrub
description: "Scrubadubdub: scrub a UI of AI hallmarks and messy spacing and text. Two passes (grep rules from the hallmark catalogue, then an eye pass on Shoot screenshots), a numbered report saying where each hallmark comes from with 1 or 2 alternatives, and fixes only the numbers the owner approves. Honours .peep/scrub.md and DESIGN.md. Use when the user runs /peep:scrub, or asks to 'de-slop', 'scrub' or 'check for AI tells' in a UI."
argument-hint: "[--full]"
label: Scrubadubdub
---

# /peep:scrub

Finds the defaults nobody chose in a UI, says where each comes from, and fixes only what the owner approves.
"formats" is `${CLAUDE_PLUGIN_ROOT}/references/formats.md`; "the catalogue" is `${CLAUDE_PLUGIN_ROOT}/references/ai-tells.md`. Read the catalogue whole before step 3; its "Never flagged" and "What not to over-flag" sections bind every step.

## 1. Read the repo setup

Read `.peep/setup.md` (formats §1). Missing: reply with exactly this and stop:

`peep: no repo setup here. Run /peep:walk first, so Scrubadubdub knows your themes and widths.`

`skills.off` lists `scrub`: §1 applies.

## 2. Scope and exceptions

- Scope: changes since the last commit on `main` (tracked changed lines plus untracked files). `--full`: the whole repo. Say which.
- Read `.peep/scrub.md` (formats §6) and the repo's `DESIGN.md` if present. Anything `DESIGN.md` declares is never flagged; drop such findings from both passes and say how many.
- Reference folders the repo names as source of truth (CLAUDE.md, DESIGN.md: e.g. `design/`) are skipped: pass each as `--skip <prefix>`.

## 3. Pass 1: grep

Run `node ${CLAUDE_PLUGIN_ROOT}/skills/scrub/scripts/scan.mjs [--full] [--skip <prefix>]...` from the repo root. It prints JSON: `findings` (file, line, id, severity, `together`, score) after inline `scrub-ignore`, earned and debt lines, and `debt` notes (a count that must drop, or was exceeded).

Impeccable installed (`.impeccable/` exists, or `npx --no-install impeccable --version` succeeds): also run `npx impeccable detect --json` on the same scope. Map each result to a catalogue ID through the `imp <name>` credit tags; a hit already in pass 1 at that file and line merges, a new one joins as a finding; one with no catalogue entry is listed once at the end as `imp:<name>`, not numbered.

## 4. Pass 2: eye

Entries marked `eye` in the catalogue need a render or a read of intent.

- Setup has a `url`: run `/peep:shoot` on the pages in scope at the setup's themes and widths. Its questions are the eye entries that could apply, each phrased pass/fail ("muted text reaches 4.5:1 on its ground"). A failed question is a finding at the file and line that styles it.
- No `url`: read the source instead and mark every eye finding `inferred`.

Eye findings take the same score rule as grep ones: catalogue severity weight (P0 3, P1 2, P2 1) plus one for each other ID in the same file; a single non-P0 hit is reported as P2.

## 5. Report

One numbered list, highest score first, grep and eye merged. Per finding, one block:

```
<n>. <file>:<line> · <id> · <severity> · <grep|eye|eye, inferred>[ · with <other ids>]
   from: <where it comes from, from the catalogue>
   try: <alternative 1>[; <alternative 2>]
```

- Alternatives come from the catalogue, made concrete for this file (a real token, face or value from the repo).
- Cluster first: say which files hold three or more IDs; that is a template, not a nit.
- Then the `debt` notes: `debt <id> · <path>: <count> -> <now>`; over the count is already a finding above.
- Never say a model made it. No praise.
- Last line exactly: `Reply with the numbers to fix (e.g. "1 3 5"), "all", "debt <n>" to log one as debt, or "none".`

Write nothing before the reply.

## 6. Fix on yes

- Fix only the numbers given. Every removal names what replaces it, in one line per number, before the edit; a fix that only deletes leaves the page flat (catalogue, the earned test).
- An alternative that needs a design choice (layout, a new accent, a type pairing): do not pick; offer `/peep:sneak <screen>` with the alternatives as its directions, and leave that number unfixed.
- `debt <n>`: add or update the `## Debt` line in `.peep/scrub.md` (formats §6), on that yes only. Earned lines are the owner's to write; offer the line, never add it.
- A debt note with fewer hits than its count: lower the count in the same change; zero deletes the line.
- Re-run both passes on the same scope and report:

```
peep scrub: fixed <numbers>; left <numbers>
<n>: <what went> -> <what replaced it>   (one line per fixed number)
[sneak: <numbers> need a design choice; run /peep:sneak <screen>]
before <n> findings, after <m>[; new: <ids>]
```

A new finding caused by a fix is fixed or reverted before the report. Nothing is staged or committed.

## Last step: run count

Bump `scrub` in `.peep/runs` (formats §1). If it was below 3, add a final line: `Why: a re-scan after the fix proves it works; a default you can name is one you can choose against.`
