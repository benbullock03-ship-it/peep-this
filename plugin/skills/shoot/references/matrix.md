# Matrix and shots

Cells: every theme in setup `themes` x every width in setup `widths`. Add reduced motion as a third axis only where the change has motion.

Per cell, in this order, because emulation carries over between calls in one browser:

1. `browser_resize` to the width, height 800.
2. `browser_emulate_media` with `colorScheme` (and `reducedMotion` where that axis applies). Set it every cell; never rely on the last one.
3. Scroll the element to the centre: `el.scrollIntoView({block: 'center'})`.
4. `browser_take_screenshot` with the element as `target`, filename `<element>-<theme>-<width>.png`.

Then one full-viewport shot per theme at the widest width, for context.

- Themes other than `light`/`dark` (e.g. a `data-theme` attribute or a class) are not a media query: set them with `browser_evaluate` the way the repo's own theme switch does, and say how.
- Shots go to `<repo>/.playwright-mcp/`; the MCP refuses paths outside the repo.
