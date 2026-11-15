---
id: AV-51
title: Global skill for Playwright element checks with pointed screenshot review
status: in-progress
depends: []
attended: true
---

# AV-51: Global skill for Playwright element checks with pointed screenshot review

**What to build:** A global skill, `~/.claude/skills/shoot/SKILL.md`, that any repo can use to check a UI change itself. It drives the page with the Playwright MCP, asserts the elements the change touched, screenshots them, then reviews each shot against a short list of questions written *before* it was taken. The goal is a review that can fail, not "looks good". The skill lives outside this repo. The ticket sits here because AV-50 is where the need showed up.

Why: `/ticket` step 3 says to "load `design-tropes` … shoot the matrix", but no `design-tropes` skill exists. The AV-50 self-test was improvised: `?demo` on port 1420, `browser_evaluate` to dump the card's HTML and check horizontal overflow, emulated light and dark screenshots. It worked, but it had to be reinvented and it hit two snags (below).

- [x] **Inputs:** a URL (or the repo's dev URL from its `CLAUDE.md`), the change in one line, and the selectors it touched. With no selectors given, the skill finds them from the diff (new or changed class names in `git diff`) and says which it picked
- [x] **Questions before shots:** for each element, write 3 to 6 pass/fail questions tied to the ticket's "Done when" and the repo's design rules (for example "chip text uses the accent token", "no text clipped mid-word", "button is the only control in the actions row"). Generic questions like "does it look good" are not allowed
- [x] **Assert in the DOM first** with `browser_evaluate`: element present, text content, computed colours of the named tokens, `scrollWidth > clientWidth` (overflow) on the element and the page, tooltip `title`s, focusability. These are facts, not impressions, and they go in the report verbatim
- [x] **Matrix:** for each element, light and dark (`browser_emulate_media`), the repo's narrow width and its normal width, and reduced motion where the repo has motion. Defaults: two widths (360 and the repo's panel width), both themes. The repo's `CLAUDE.md` can override them
- [x] **Pointed shots:** element screenshots (`target` = selector), with one full-viewport shot per theme for context. Scroll the element to the centre first
- [x] **Review:** read every shot and answer each question pass, fail, or can't tell, with a one-line reason pointing at what is visible. Any fail is fixed or named before the work is called done. "Can't tell" turns into a DOM assertion next time
- [x] **Report:** a table of element × matrix cell × question → verdict, the DOM facts, and the shot paths. Under 30 lines. No praise
- [x] **Housekeeping:** shots go to `<repo>/.playwright-mcp/` (the only writable root besides the repo: the scratchpad was refused in AV-50). Delete the folder at the end unless the user asks to keep it, and never stage it. Close the browser
- [x] **Hand-off:** owner-only checks (native toasts, real window focus, OS dialogs, anything outside the page) are listed as owed, never claimed
- [x] Point `/ticket` step 3 and `~/.claude/CLAUDE.md`'s front-end section at `shoot` in place of the missing `design-tropes`
- [x] Prove it on this repo: rerun the AV-50 question card check through the skill and compare it with what was found by hand

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

- **`/ticket` keeps `design-tropes` as "the repo's design skill if it has one"** rather than deleting it: it exists as a repo skill in datamapuk. `shoot` replaces only the "shoot the matrix" step.
- **Contrast added to the DOM asserts** after the proof run: the one fail below was "can't tell" from the screenshot and settled only by the computed ratio.
- **Widths 360 and 1280.** This repo's `CLAUDE.md` names no panel width; the card is a fixed 354px, so the width axis here only tests page overflow.

**Not taken:** visual regression baselines, cross-browser runs, E2E flows across pages (this repo's `CLAUDE.md` says no E2E).

## Proof run 2026-11-15

`/shoot http://localhost:1420/?demo`, research-02's question card, dark and light at 360 and 1280. Selector `.ps-item:has(.card[data-tier="question"])`.

| Question | Verdict | Fact |
|---|---|---|
| Head reads `AskUserQuestion`, `1 of 2`, `question` | pass | head children exactly those three |
| Chip uses `--ps-accent` | pass | `rgb(247,214,74)` dark, `rgb(138,90,0)` light = token |
| Question text full, wrapped | pass | `pre-wrap`, no overflow at either width |
| One button, `Answer in terminal` | pass | 1 button; `.card-deny`/`.card-reason`/`.card-countdown` absent |
| No page overflow | pass | false at 360 and 1280 |
| Option `title` = description | pass | both match the demo descriptions |
| Options inert | pass | `SPAN`, tabIndex -1, `cursor: default`, border `--ps-line`, text `--ps-muted` |
| No option label clipped | pass | both false |
| Bubble: question text, clipped, waiting fill | pass | bg `--st-waiting`, ellipsis after line 2 |
| **Light: chip text readable (AA)** | **fail** | `.card-tier` 4.40:1 at 10px on `rgb(230,221,205)`; needs 4.5. Dark 13.52 |

Other light ratios: muted target and options 4.71, button 5.93, bubble 5.05. Compared with AV-50 by hand: same passes, plus the contrast fail that the hand check missed. The fail is the shared light `--ps-accent` on the card's desk background, so it hits every tier chip, not only `question`. Not fixed here: changing a design token is the owner's call. Shots deleted, browser closed, `git status` clean apart from the existing `reports/`. `.gitignore` has no `.playwright-mcp/` entry.

Owed to the owner (outside the page): toast, `Answer in terminal` focusing a real terminal, hotkey inbox.

## Owed before done

- Owner agrees the light chip contrast fail (4.40:1) is a real fail, and says whether to fix it (new ticket) or accept it.
