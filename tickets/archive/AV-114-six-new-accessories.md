---
id: AV-114
title: Six new accessories for earned and supporter cosmetics
status: closed
depends: []
attended: true
---

# AV-114 — Six new accessories for earned and supporter cosmetics

**What to build:** Vendor the owner's new `peep.js` carrying six accessories: five rhythm milestones and one supporter.

- [x] Owner draws: first Pack Up, first Ducks with 0 loose, 10 tickets closed, first Coderot with 0 survivors, first architecture candidate acted on, supporter
- [x] Replace `design/peep.js`, `src/vendor/peep.js` and `plugin/skills/goose/assets/peep.js`, byte-identical
- [x] The six are excluded from the identity hash (the hashed set stays the 14)
- [x] Vitest: hash output unchanged for existing cwds

**Done when:** all three copies match, the six render on a test page, and no existing project's hashed look changed.

## Prompt

Attach: `design/peep.js`, `design/shell/` (SHELL-SPEC.md, shell.css, demo.html), `design/skill-pack-2/tokens.css`.

> Add six accessories to Peep This's `<peep-bot>` in `peep.js`, drawn exactly like the existing 13 (hardhat to wizard): same 32x32 grid, same `R()` / `rect()` / `set()` calls with head (`hx,hy`), face (`fx,fy`) and body (`bx,by`, `P.sq`) offsets, colours from the `C` table (list any new entries), and `ax`/`ay` set so the antenna still clears the accessory. Every pose, all five palette colours, readable at 32px, on dark `#141218` and light. Change nothing else in the file: no redrawn body, no changed accessory, no new poses; only new `else if` branches and the header comment's accessory list.
>
> These are earned cosmetics, badges for a working rhythm, so each should feel like a small trophy rather than a costume. Keep the ids; motifs are suggestions:
> 1. `packed`: first Pack Up (a tiny backpack strap or bindle)
> 2. `clear`: first Ducks with 0 loose ends (a rubber duck sitting on the head)
> 3. `ten`: 10 tickets closed (a ribbon medal on the chest)
> 4. `spotless`: first Coderot with 0 survivors (a sparkle or broom pin)
> 5. `architect`: first architecture candidate acted on (a pencil behind the ear or a rolled blueprint; not a hat, `hardhat` exists)
> 6. `supporter`: sponsor (a small heart badge; warm, not premium-gold, distinct from `crown` and `halo`)
>
> Deliver the full updated `peep.js` and one preview page: the six x three poses (idle, working, success) x five colours at 96px, plus a 32px row, dark and light.

## Done 2026-11-16

- Owner confirmed the uncommitted `peep.js` (new ids `packed | clear | ten | spotless | architect | supporter`, new colours `pack`, `packS`, `shineS`) is final; all three copies byte-identical.
- `EARNED_ACCESSORIES` in `src/identity.ts`; the hash still uses the 14, pinned outputs unchanged, new test `never hashes to an earned accessory`.
- `?accessories` page now shows the six; all render in idle, waiting and error (`.playwright-mcp/av114-acc.png`).
