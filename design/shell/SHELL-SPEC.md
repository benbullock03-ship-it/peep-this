# Peep This — new shell: handoff for Claude Code

The chosen design is **Pixel desktop**: a 90s desktop-pet window with hard edges, a striped title bar and square speech bubbles. It is working plain HTML + CSS + JS. Open `demo.html` in a browser to see the open window and the minimised strip side by side, in dark and light. `ref-C-pixel.png` is a screenshot.

## Files

| File | What it is |
|---|---|
| `shell.css` | All shell styling. Tokens at the top, shared rules, then the pixel look. |
| `shell.js` | Renders the window and strip from an agent list. Exposes `PeepShell.mount`. |
| `peep.js` | The `<peep-bot>` component, including the newer accessories. |
| `demo.html` | Demo page only. Its inline `<style>` is page chrome, not part of the shell. |

## What the design does

- **Two modes.** `window`: 360px wide, the open view. `strip`: 62px wide, docks on the right screen edge, shows peeps only (no names, no usage).
- **Speech bubbles.** Each peep says what it is doing (`say`) instead of a plain status line. Attention states (`waiting`, `error`, `stuck`) fill the bubble with the state colour.
- **Habitat.** Each peep stands on a striped shelf plank.
- **Attention first.** Agents in `waiting`, `error` or `stuck` sort to the top; the rest keep their order. In the strip they get a tinted, ringed cell.
- **Detail view.** Clicking a row opens it in place: 96px peep on a stage with a larger bubble, then tags (time in state, model, kind), last tool, a context meter and the cwd. Clicking the header closes it. One open at a time.
- **Usage.** 5h and 7d sit small in the header, each with a mini bar.
- **Themes.** Dark is default. Set `data-theme="light"` on `<html>` for light. Every colour is a CSS variable in `:root`.

## Using it

```html
<link rel="stylesheet" href="shell.css">
<script src="peep.js"></script>
<script src="shell.js"></script>
<div id="win"></div>
<script>
  const shell = PeepShell.mount(document.getElementById('win'), {
    mode: 'window',          // 'window' | 'strip'
    agents: [...],           // see shape below; omit to use sample data
    usage: { fiveHour: 48, sevenDay: 15 }
  });
  shell.update(newAgents, newUsage);   // call whenever sessions change
</script>
```

Agent shape:

```js
{ name: 'datamapuk-3e', state: 'thinking',        // idle thinking working waiting success error stuck
  color: 'lavender', acc: 'crown',                // peep colour + accessory
  say: 'Hmm… how should the JSON-LD nest?',       // speech bubble text
  cwd: 'C:\\Projects\\datamapuk', model: 'Opus 5.5',
  tool: 'Agent S-309 JSON-LD structured data',    // last tool
  since: '38s', ctx: 18, kind: 'interactive · cli' }
```

Events dispatched on the mount element (all bubble):

- `peepshell:minimise` — the `›` button in the window header. Switch to the strip.
- `peepshell:expand` — the `‹` button on the strip. Switch to the window.
- `peepshell:select` — a peep clicked in the strip; `event.detail` is the agent. Suggested behaviour: expand the window with that agent open.

## What you need to wire up

1. **`say` text.** The existing widget has state and last tool but no sentence. Generate one from them, e.g. working → "Running {tool}…", waiting → the permission prompt, error → the error summary, stuck → "zzz… no reply for {since}", success → "All done!".
2. **Window ↔ strip switching** and docking the strip to the right screen edge. The demo just shows both side by side.
3. **Live data.** Replace `SAMPLE_AGENTS` / `SAMPLE_USAGE`; call `update()` on change. `update()` re-renders the whole list, which restarts peep animations. If that flickers with frequent updates, diff by agent name and only touch changed rows.
4. **Theme.** Follow the OS setting or add a toggle; set `data-theme` on `<html>`.
5. **Fonts.** Fredoka (title), IBM Plex Sans (UI), IBM Plex Mono (names, paths). Loaded from Google Fonts in the demo; bundle them for a desktop app.

## Notes

- Peeps render crisp at multiples of 32px. Sizes used: 40 (strip), 48 (rows), 96 (detail). The non-multiples are slightly soft; switch to 32/64/96 if that bothers you.
- Rows and strip cells are real `<button>`s with `aria-expanded` / `aria-label`.
- Not designed yet: empty state (no agents), more than about 8 agents in the strip (it will need scrolling), and any notification outside the widget.
