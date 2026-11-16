---
name: shoot
description: "Shoot: check a web UI change (or a desktop app's webview) in a real browser with DOM asserts and screenshots judged against questions written first. Use when the user runs /peep:shoot, after a UI change before calling it done, or asks to 'check it in the browser'."
argument-hint: "[change in one line] [selectors...]"
label: Shoot
---

# /peep:shoot: a UI review that can fail

Verdicts and facts, not impressions. No praise anywhere in the report.

## 1. Setup

Preamble `${CLAUDE_PLUGIN_ROOT}/references/preamble.md` (setup, `skills.off`, self-setup, run count). Take `url`, `dev`, `themes` (default `light, dark`), widths from `shoot.widths`, else `widths` (default `375, 1280`).

- **No `url`:** reply with exactly `Shoot doesn't apply: no url in .peep/setup.md (Shoot is for web UIs only).`, nothing else, and stop. Do not guess a URL or open a browser.
- **Desktop webview** (the change is in a Tauri, WebView2 or Electron app's page, not a browser tab): follow `references/webview.md` instead of the Playwright MCP for DOM facts and shots; every other step is unchanged.
- **No Playwright MCP tools** (`browser_navigate` etc.) and not a desktop webview: reply `Shoot needs the Playwright MCP; it isn't connected.` and stop.
- Check the URL is serving (`curl -s -o /dev/null -w "%{http_code}" <url>`). If not and `dev` is set, start `dev` in the background and poll until it serves (max about 60s). If not and no `dev`, say so and stop.

## 2. Change and selectors

- **Change:** one line, from the arguments, the ticket in progress, or the user.
- **Selectors:** from the arguments, else from `git diff` (staged and unstaged, or the last commit if clean): new or changed class names and ids. Say which you picked.

## 3. Questions, before any shot

Per element, 3 to 6 pass/fail questions tied to the ticket's "Done when" and the repo's own design rules (its docs or design tokens). E.g. "chip text uses the accent token", "no text clipped mid-word". Banned: "does it look good", "is it clean", "is it consistent" with nothing named. Post them as a numbered list in a message of their own before any `browser_*` call; putting them only in the final table does not count. Answer exactly these later; a question added after a shot is marked `(added)`.

## 4. DOM, then shots

Gate: the message that makes the first `browser_*` call must begin with a visible line `Questions:` followed by the step 3 list, unless an earlier message already showed it. Thinking does not count, and terse output modes do not waive it.

- DOM facts per width with `browser_evaluate`: `references/dom-asserts.md`. A screenshot cannot overrule a DOM fact.
- The theme x width matrix and shot order: `references/matrix.md`.
- Before every fact and shot: `references/stable.md`. A verdict that changes on a rerun is `flaky`, not `pass`.

## 5. Review

Read every shot. Answer each question `pass`, `fail`, `flaky` or `can't tell`, one line pointing at what is visible. A DOM fact answers first where one exists. Any `fail` is fixed and re-shot, or named as open, before the work is called done. Each `can't tell` names the DOM assertion to add next run.

## 6. Report

The final reply is this and nothing else, under 30 lines (no summary before it; notes such as a missing `.gitignore` entry or a console error go in one `Notes:` line at the end):

```
Shoot: <change>, <url>
| element | theme/width | question | verdict | reason |
DOM facts: <verbatim values>
Shots: <paths, or "deleted">
Owed to the owner: <anything outside the page: native dialogs, OS notifications, window focus, hotkeys; or "none">
```

Never claim what is owed.

## 7. Housekeeping

`browser_close`. Stop a dev server and every process this run started; restart any copy of the app it stopped (`references/webview.md`). Delete `.playwright-mcp/` unless the user asked to keep the shots; never stage it. If `.gitignore` lacks it, say so rather than editing it. `git status --short` shows nothing new from this run.

## 8. Last step: run count

Per the preamble, after the report: `Why: a shot you checked proves it works; reading the code does not.`

Not here: pixel-diff baselines, cross-browser runs, multi-page E2E flows, native UI with no web page behind it.
