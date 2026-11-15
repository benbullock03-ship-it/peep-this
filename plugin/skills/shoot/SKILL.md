---
name: shoot
description: "Shoot: check a web UI change in a real browser. Asserts the touched elements in the DOM, screenshots them across the repo setup's themes and widths with the Playwright MCP, and judges each shot against pass/fail questions written first. Web UIs only. Use when the user runs /peep:shoot, after a UI change before calling it done, or asks to 'check it in the browser' or 'screenshot it'."
argument-hint: "[change in one line] [selectors...]"
label: Shoot
---

# /peep:shoot: a UI review that can fail

Verdicts and facts, not impressions. No praise anywhere in the report.

## 1. Setup

Read `.peep/setup.md` (`${CLAUDE_PLUGIN_ROOT}/references/formats.md` §1): `url`, `dev`, `themes` (default `light, dark`), `widths` (default `375, 1280`).

- **No `url`:** reply with exactly `Shoot doesn't apply: no url in .peep/setup.md (Shoot is for web UIs only).`, nothing else, and stop. Do not guess a URL or open a browser.
- **No Playwright MCP tools** (`browser_navigate` etc.): reply `Shoot needs the Playwright MCP; it isn't connected.` and stop.
- Check the URL is serving (`curl -s -o /dev/null -w "%{http_code}" <url>`). If not and `dev` is set, start `dev` in the background and poll until it serves (max about 60s). If not and no `dev`, say so and stop.

## 2. Change and selectors

- **Change:** one line, from the arguments, the ticket in progress, or the user.
- **Selectors:** from the arguments, else from `git diff` (staged and unstaged, or the last commit if clean): new or changed class names and ids. Say which you picked.

## 3. Questions, before any shot

Per element, 3 to 6 pass/fail questions tied to the ticket's "Done when" and the repo's own design rules (its docs or design tokens). E.g. "chip text uses the accent token", "no text clipped mid-word". Banned: "does it look good", "is it clean", "is it consistent" with nothing named. Write them into the reply before the first shot.

## 4. DOM, then shots

- DOM facts per width with `browser_evaluate`: `references/dom-asserts.md`. A screenshot cannot overrule a DOM fact.
- The theme x width matrix and shot order: `references/matrix.md`.

## 5. Review

Read every shot. Answer each question `pass`, `fail` or `can't tell`, one line pointing at what is visible. A DOM fact answers first where one exists. Any `fail` is fixed and re-shot, or named as open, before the work is called done. Each `can't tell` names the DOM assertion to add next run.

## 6. Report

Under 30 lines, this shape:

```
Shoot: <change>, <url>
| element | theme/width | question | verdict | reason |
DOM facts: <verbatim values>
Shots: <paths, or "deleted">
Owed to the owner: <anything outside the page: native dialogs, OS notifications, window focus, hotkeys; or "none">
```

Never claim what is owed.

## 7. Housekeeping

`browser_close`. Stop a dev server this run started. Delete `.playwright-mcp/` unless the user asked to keep the shots; never stage it. If `.gitignore` lacks it, say so rather than editing it. `git status --short` shows nothing new from this run.

Not here: pixel-diff baselines, cross-browser runs, multi-page E2E flows, native apps.
