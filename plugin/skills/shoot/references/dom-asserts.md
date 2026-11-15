# DOM asserts

One `browser_evaluate` per element, once per width (layout can change). Collect as facts:

- present: count of matches; visible: `getBoundingClientRect()` width and height > 0.
- `textContent`, trimmed.
- Computed `color`, `background-color`, `border-color` for the tokens the questions name. Resolve a token with `getComputedStyle(document.documentElement).getPropertyValue('--token')` and compare.
- Contrast: WCAG ratio of each text element's computed `color` against its effective background (its own, else the nearest ancestor's non-transparent one), with `font-size` and `font-weight`. Under 4.5 for text below 18px (14px bold) is a fail. Eyes cannot judge 4.4 against 4.5.
- Overflow: `el.scrollWidth > el.clientWidth` on the element; `document.documentElement.scrollWidth > innerWidth` on the page.
- `title` on anything that should have a tooltip.
- Focusability: `tabIndex`, tag, `disabled`; `cursor` where something must look inert.

Report values verbatim (`rgb(247, 214, 74)`, `4.62`), not "looks yellow".
