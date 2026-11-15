---
id: AV-51
title: Global skill for Playwright element checks with pointed screenshot review
status: todo
depends: []
attended: true
---

# AV-51: Global skill for Playwright element checks with pointed screenshot review

**What to build:** A global skill, `~/.claude/skills/shoot/SKILL.md`, that any repo can use to check a UI change itself. It drives the page with the Playwright MCP, asserts the elements the change touched, screenshots them, then reviews each shot against a short list of questions written *before* it was taken. The goal is a review that can fail, not "looks good". The skill lives outside this repo. The ticket sits here because AV-50 is where the need showed up.

Why: `/ticket` step 3 says to "load `design-tropes` … shoot the matrix", but no `design-tropes` skill exists. The AV-50 self-test was improvised: `?demo` on port 1420, `browser_evaluate` to dump the card's HTML and check horizontal overflow, emulated light and dark screenshots. It worked, but it had to be reinvented and it hit two snags (below).

- [ ] **Inputs:** a URL (or the repo's dev URL from its `CLAUDE.md`), the change in one line, and the selectors it touched. With no selectors given, the skill finds them from the diff (new or changed class names in `git diff`) and says which it picked
- [ ] **Questions before shots:** for each element, write 3 to 6 pass/fail questions tied to the ticket's "Done when" and the repo's design rules (for example "chip text uses the accent token", "no text clipped mid-word", "button is the only control in the actions row"). Generic questions like "does it look good" are not allowed
- [ ] **Assert in the DOM first** with `browser_evaluate`: element present, text content, computed colours of the named tokens, `scrollWidth > clientWidth` (overflow) on the element and the page, tooltip `title`s, focusability. These are facts, not impressions, and they go in the report verbatim
- [ ] **Matrix:** for each element, light and dark (`browser_emulate_media`), the repo's narrow width and its normal width, and reduced motion where the repo has motion. Defaults: two widths (360 and the repo's panel width), both themes. The repo's `CLAUDE.md` can override them
- [ ] **Pointed shots:** element screenshots (`target` = selector), with one full-viewport shot per theme for context. Scroll the element to the centre first
- [ ] **Review:** read every shot and answer each question pass, fail, or can't tell, with a one-line reason pointing at what is visible. Any fail is fixed or named before the work is called done. "Can't tell" turns into a DOM assertion next time
- [ ] **Report:** a table of element × matrix cell × question → verdict, the DOM facts, and the shot paths. Under 30 lines. No praise
- [ ] **Housekeeping:** shots go to `<repo>/.playwright-mcp/` (the only writable root besides the repo: the scratchpad was refused in AV-50). Delete the folder at the end unless the user asks to keep it, and never stage it. Close the browser
- [ ] **Hand-off:** owner-only checks (native toasts, real window focus, OS dialogs, anything outside the page) are listed as owed, never claimed
- [ ] Point `/ticket` step 3 and `~/.claude/CLAUDE.md`'s front-end section at `shoot` in place of the missing `design-tropes`
- [ ] Prove it on this repo: rerun the AV-50 question card check through the skill and compare it with what was found by hand

**Done when:** `/shoot` on this repo's `?demo` page checks the AV-50 question card across both themes and two widths, reports DOM facts plus a verdict per question, leaves no stray files, and the owner agrees its fails are real fails.

## Notes from AV-50 (2026-11-15)

- Playwright MCP writes only under the repo or `.playwright-mcp/`. A scratchpad path was refused with "outside allowed roots".
- `npm run dev` with `?demo` gives mock sessions with no Tauri. The dev server may already be up on 1420: check before starting one.
- Dark mode needs `browser_emulate_media({ colorScheme: "dark" })`, and an earlier emulation carries over to later shots in the same browser, so set the scheme for every cell.
- Tauri-only behaviour (toasts, `focus_session`, the global hotkey) cannot be seen from the browser. Those checks are owed to the owner.

## Decisions taken without a ruling

- **Name `shoot`**, a global skill (not a repo skill), because the need is the same in every front-end repo. `/ticket` keeps its step and points here.
- **Questions are written before the shots**, so reading the screenshots can't drift into confirming what was expected.
- **DOM assertions come before screenshots.** Overflow, colours and text are checked as numbers and strings. The screenshot judges only what a human would notice.
- **No pixel-diff baselines.** No stored golden images to maintain. Add them later only if a repo wants regression checks.
- **No new dependency:** the Playwright MCP that is already installed, with no `@playwright/test` in repos.

**Not taken:** visual regression baselines, cross-browser runs, E2E flows across pages (this repo's `CLAUDE.md` says no E2E).
