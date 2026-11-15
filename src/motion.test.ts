// @ts-expect-error type error without @types/node package
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { MOTION_FRAMES, MOTION_MS, motionMs } from "./motion";

// Vitest blanks CSS imports, `?raw` included, so read the file.
const css: string = readFileSync(new URL("./styles.css", import.meta.url), "utf8");

describe("motion", () => {
  it("timers scale with motion units and drop to 0 under reduced motion", () => {
    expect(motionMs(1, false)).toBe(MOTION_MS);
    expect(motionMs(3, false)).toBe(3 * MOTION_MS);
    expect(motionMs(3, true)).toBe(0);
  });

  it("CSS tokens match the TS constants", () => {
    expect(css).toContain(`--motion-ms: ${MOTION_MS}ms;`);
    expect(css).toContain(`--motion-steps: steps(${MOTION_FRAMES}, end);`);
  });

  it("no smooth easing left", () => {
    expect(css.length).toBeGreaterThan(1000);
    const smooth = css.match(/\b(ease|ease-in|ease-out|ease-in-out|cubic-bezier\([^)]*\))\b/g) ?? [];
    expect(smooth).toEqual([]);
  });
});
