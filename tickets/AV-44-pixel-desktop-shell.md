---
id: AV-44
title: Pixel desktop shell for the open window
status: todo
depends: [AV-26]
attended: false
---

# AV-44 — Pixel desktop shell for the open window

**What to build:** The owner's "Pixel desktop" design (`design/shell/`: `SHELL-SPEC.md`, `shell.css`, `shell.js`, `demo.html`, `ref-C-pixel.png`) applied to the live open window (spec §12.0). The design is the reference: port its markup and CSS into the app's TS modules, keep its look, wire it to the real snapshot. Strip mode is AV-39; light theme is AV-45; window transparency is AV-35.

Owner rulings 2026-11-14: peep sizes 48 (rows), 96 (open stage) as designed; AV-36's 10-block bars stay (mini in the header, full on the open row) instead of the design's smooth bars; square chrome with 6px hard shadow replaces the stair corners.

- [ ] Card: square, 2px border in the text colour, 6px hard shadow, mono type, inside today's window (transparency is AV-35)
- [ ] Header: striped title bar, "PEEP THIS" title, 5h/7d as mini AV-36 block bars, existing header controls (settings button from AV-26) restyled as the design's square icon buttons. The `›` minimise button is left out until AV-39
- [ ] Rows: 48px peep on a striped plank, speech bubble (2-line clamp), session name, outlined state chip. Attention states fill the bubble with the state colour
- [ ] `say` text derived from state and data in a pure module: working "Running {tool}…", waiting the permission prompt/notification text, error the error summary, stuck "zzz… no reply for {since}", success "All done!" or the result summary, thinking the task or a thinking line, idle a resting line
- [ ] Sort: attention states first, otherwise the existing order
- [ ] Open row in place, one at a time: 96px stage with a larger bubble (4-line clamp), tags (time in state, model, kind), last tool, context as a block bar, cwd, then AV-26's last-5 trail. Clicking the head closes it
- [ ] AV-23's approval card still mounts under its row (closed or open) and works with keyboard and mouse as before; pulse on entering `waiting` kept
- [ ] Rows diffed by session id so a snapshot does not restart peep animations or close the open row
- [ ] Dark tokens from `shell.css` `:root` as CSS variables; existing accent and fonts stay bundled locally
- [ ] Demo mode (`src/demo.ts`, browser `npm run dev`) shows the new shell
- [ ] Vitest: `say` derivation per state, attention sort

**Done when:** the owner saw the new shell in `npm run tauri dev` with live sessions, opened a row, and accepted the look.
