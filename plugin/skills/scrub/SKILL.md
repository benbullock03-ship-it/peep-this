---
name: scrub
description: "Scrubadubdub: scrub a UI of AI hallmarks and messy spacing and text. Two passes (grep rules from the hallmark catalogue, then an eye pass on Shoot screenshots), a numbered report saying where each hallmark comes from with 1 or 2 alternatives in the repo's own tokens, and fixes only the numbers the owner approves. Honours .peep/scrub.md and DESIGN.md. Use when the user runs /peep:scrub, or asks to 'de-slop', 'scrub' or 'check for AI tells' in a UI."
argument-hint: "[--full]"
label: Scrubadubdub
---

# /peep:scrub

Finds the defaults nobody chose in a UI, teaches where each comes from, and fixes only what the owner approves. Its aim is a UI that looks chosen, not a UI with hallmarks deleted.
"formats" is `${CLAUDE_PLUGIN_ROOT}/references/formats.md`; "the catalogue" is `${CLAUDE_PLUGIN_ROOT}/references/ai-tells.md`. Read the catalogue whole before step 3. Its "Never flagged" and "What not to over-flag" sections bind every step. Files in `references/` beside this one are read only when their step runs.

## 1. Read the repo setup

Read `.peep/setup.md` (formats §1). Missing: reply with exactly this and stop:

`peep: no repo setup here. Run /peep:walk first, so Scrubadubdub knows your themes and widths.`

`skills.off` lists `scrub`: §1 applies.

## 2. Scope and the house style

- Scope: changes since the last commit on `main` (changed lines plus untracked files). `--full`: the whole repo. Say which.
- Read `.peep/scrub.md` (formats §6) and `DESIGN.md` if present. Anything `DESIGN.md` declares is never flagged. Drop such findings from both passes and say how many were dropped.
- Reference folders the repo names as source of truth (in CLAUDE.md or DESIGN.md, e.g. `design/`) are skipped: pass each one as `--skip <prefix>`.
- **House style:** note the repo's tokens (`:root` custom properties, a Tailwind theme, DESIGN.md), its faces, its spacing scale, and its reference design. Every alternative and every eye question is phrased in these. No tokens at all: say so once, and make the first alternative "add a token".

## 3. Pass 1: grep

Run `node ${CLAUDE_PLUGIN_ROOT}/skills/scrub/scripts/scan.mjs [--full] [--skip <prefix>]...` from the repo root. It prints JSON: `scanned` (number of UI files read), `findings` (one per ID per file: `file`, first `line`, all `lines`, `id`, `severity`, `together`, `score`) and `debt` notes. Inline `scrub-ignore`, earned and debt lines are already applied.

`scanned` is 0 in diff scope: reply `peep scrub: no UI changes since <base>. Run /peep:scrub --full for the whole repo.` and stop. This counts as a run.

Impeccable installed (`.impeccable/` exists, or `npx --no-install impeccable --version` succeeds): also run `npx impeccable detect --json` on the same scope. Map each result to a catalogue ID through the `imp <name>` credit tags. A result at a file and line pass 1 already found merges with it, and a new one joins as a finding. A result with no catalogue entry is listed once at the end as `imp:<name>`, without a number.

## 4. Pass 2: eye

Follow `references/eye.md`: the catalogue's `eye` entries, spacing and text included, judged on Shoot screenshots, or `inferred` from the source when there is no `url`.

## 5. Report

One numbered list, highest score first, with grep and eye findings merged:

```
<n>. <file>:<line>[ (+<k> more: <lines>)] · <id> · <severity> · <grep|eye|eye, inferred>[ · with <other ids>]
   from: <where it comes from, from the catalogue>
   try: <alternative 1 in this repo's tokens>[; <alternative 2>]
```

- Open with a cluster line for each file holding three or more IDs: `cluster <file>: <ids>. One root: <the shared origin>`. A cluster is a template, not a nit.
- Then the `debt` notes, each as `debt <id> · <path>: <count> -> <now>`. Over the count is already a finding above.
- Never say a model made it. No praise.
- Last line exactly: `Reply with numbers to fix ("1 3 5" or "all"), "debt <n>" to log one, "wrong <n>" if one is not a hallmark here, or "none".`

Write nothing before the reply.

## 6. On the reply

Follow `references/fix.md`.

## Last step: run count

Bump `scrub` in `.peep/runs` (formats §1). If it was below 3, add a final line: `Why: a re-scan after the fix proves it works; a default you can name is one you can choose against.`
