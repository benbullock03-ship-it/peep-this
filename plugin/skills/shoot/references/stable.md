# Stable shots

A check that can give a different verdict on a second run proves nothing. Before each shot:

- **Wait on a condition, not a clock.** Poll a DOM fact (element present, text set, `aria-busy` gone) every 100 to 250ms up to a timeout; never a bare `sleep`. A timeout is a `fail` with the fact that never arrived.
- **Fonts.** `await document.fonts.ready`, then check the computed `font-family` of the element is the intended face, not a fallback.
- **Motion.** Static questions: emulate `reducedMotion: reduce`, or inject `*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}`. Motion under test: shoot named frames by pausing (`el.getAnimations().forEach(a => { a.pause(); a.currentTime = <ms>; })`), never a random moment.
- **Canvas and rAF loops** draw a different frame per shot. Ask only frame-independent questions of them (size, position, presence), or freeze them through the component's own API.
- **Clocks and live data.** Relative times, counters and live feeds change between shots. Use fixture data, or assert them by pattern (`/^\d+m$/`) in the DOM and leave them out of visual questions.
- **Pointer and focus.** Move the pointer to a corner and blur `document.activeElement` before each shot, unless hover or focus is the subject.
- **Viewport.** Fixed width and height per cell and one device scale factor; set them every cell. Hide scrollbars or keep them on in every cell, never mixed.
- **State between cells.** Reload when a theme or setting is stored by the page (localStorage, a class on `<html>`), so one cell cannot leak into the next.
- **Retry once, with a reason.** A failing shot is retaken once only after naming a cause (still loading, wrong cell). Pass then fail on the same code is reported `flaky`, never `pass`; name what moved.
- **Facts over pixels.** Anything a DOM fact can answer is answered by it; a shot answers only what needs eyes.
