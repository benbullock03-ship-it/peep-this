/** Calls `apply` with the whole-pixel content height, skipping repeats. */
export function createHeightReporter(apply: (height: number) => void): (height: number) => void {
  let last = -1;
  return (height) => {
    const h = Math.ceil(height);
    if (!Number.isFinite(h) || h <= 0 || h === last) return;
    last = h;
    apply(h);
  };
}

/** Reports header + list natural height (CSS px) whenever either changes size. */
export function watchContentHeight(parts: HTMLElement[], apply: (height: number) => void): () => void {
  const report = createHeightReporter(apply);
  let frame = 0;
  const measure = () => {
    frame = 0;
    report(parts.reduce((sum, el) => sum + el.getBoundingClientRect().height, 0));
  };
  const ro = new ResizeObserver(() => {
    if (!frame) frame = requestAnimationFrame(measure);
  });
  parts.forEach((el) => ro.observe(el));
  return () => {
    ro.disconnect();
    if (frame) cancelAnimationFrame(frame);
  };
}
