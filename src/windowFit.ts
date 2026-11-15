/** Calls `apply` with the whole-pixel length, skipping repeats of the same length under the same tag. */
export function createHeightReporter(apply: (height: number, tag: string) => void): (height: number, tag?: string) => void {
  let last = "";
  return (height, tag = "") => {
    const h = Math.ceil(height);
    if (!Number.isFinite(h) || h <= 0) return;
    const key = `${tag}:${h}`;
    if (key === last) return;
    last = key;
    apply(h, tag);
  };
}

export type Measured = { length: number; tag: string };

/** Reports `measure()` (CSS px, null to skip) whenever a part changes size; the result asks for a fresh measure. */
export function watchContent(parts: HTMLElement[], measure: () => Measured | null, apply: (length: number, tag: string) => void): () => void {
  const report = createHeightReporter(apply);
  let frame = 0;
  const run = () => {
    frame = 0;
    const m = measure();
    if (m) report(m.length, m.tag);
  };
  const schedule = () => {
    if (!frame) frame = requestAnimationFrame(run);
  };
  const ro = new ResizeObserver(schedule);
  parts.forEach((el) => ro.observe(el));
  return schedule;
}
