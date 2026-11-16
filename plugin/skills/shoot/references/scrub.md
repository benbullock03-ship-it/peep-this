# `--scrub`: the hallmark pass

Runs after the normal Shoot report (step 6) has been printed. Finds the defaults nobody chose in a UI, teaches where each comes from, and fixes only what the owner approves. The aim is a UI that looks chosen, not one with hallmarks deleted.
"formats" is `${CLAUDE_PLUGIN_ROOT}/references/formats.md`; "the catalogue" is `${CLAUDE_PLUGIN_ROOT}/references/ai-tells.md`. Read the catalogue whole first. Its "Never flagged" and "What not to over-flag" sections bind every step. `eye.md` and `fix.md` beside this file are read only when their step runs.

## Scope and the house style

- Scope: changes since the last commit on `main` (changed lines plus untracked files). `--full` beside `--scrub`: the whole repo. Say which.
- Read `.peep/scrub.md` (formats §6) and `DESIGN.md` if present. Anything `DESIGN.md` declares is never flagged. Drop such findings from both passes and say how many were dropped.
- Reference folders the repo names as source of truth (in CLAUDE.md or DESIGN.md, e.g. `design/`) are skipped: pass each one as `--skip <prefix>`.
- **House style:** note the repo's tokens (`:root` custom properties, a Tailwind theme, DESIGN.md), its faces, its spacing scale, and its reference design. Every alternative and every eye question is phrased in these. No tokens at all: say so once, and make the first alternative "add a token".

## Pass 1: grep

Run `node ${CLAUDE_PLUGIN_ROOT}/skills/shoot/scripts/scan.mjs [--full] [--skip <prefix>]...` from the repo root. It prints JSON: `scanned` (UI files read), `findings` (one per ID per file: `file`, first `line`, all `lines`, `id`, `severity`, `together`, `score`) and `debt` notes. Inline `scrub-ignore`, earned and debt lines are already applied.

`scanned` is 0 in diff scope: print `peep scrub: no UI changes since <base>. Run /peep:shoot --scrub --full for the whole repo.` and stop the pass.

Impeccable installed (`.impeccable/` exists, or `npx --no-install impeccable --version` succeeds): also run `npx impeccable detect --json` on the same scope. Map each result to a catalogue ID through the `imp <name>` credit tags. A result at a file and line pass 1 already found merges with it, and a new one joins as a finding. A result with no catalogue entry is listed once at the end as `imp:<name>`, without a number.

## Pass 2: eye

Follow `eye.md`, on the shots this run already took.

## Findings

One numbered list, highest score first, grep and eye merged:

```
<n>. <file>:<line>[ (+<k> more: <lines>)] · <id> · <severity> · <grep|eye|eye, inferred>[ · with <other ids>]
   from: <where it comes from, from the catalogue>
   try: <alternative 1 in this repo's tokens>[; <alternative 2>]
```

- Open with a cluster line for each file holding three or more IDs: `cluster <file>: <ids>. One root: <the shared origin>`. A cluster is a template, not a nit.
- Then the `debt` notes, each as `debt <id> · <path>: <count> -> <now>`. Over the count is already a finding above.
- Never say a model made it. No praise.
- Last line exactly: `Reply with numbers to fix ("1 3 5" or "all"), "debt <n>" to log one, "wrong <n>" if one is not a hallmark here, or "none".`

Write nothing before the reply. On the reply, follow `fix.md`.
