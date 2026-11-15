/** Mirrors `--motion-ms` / `--motion-frames` in styles.css and `motion.rs` (spec §12.3). */
export const MOTION_MS = 160;
export const MOTION_FRAMES = 5;

export function reducedMotion(): boolean {
  return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** How long a JS timer waits for a CSS motion of `frames` motion units; 0 under reduced motion. */
export function motionMs(units = 1, reduced = reducedMotion()): number {
  return reduced ? 0 : MOTION_MS * units;
}
