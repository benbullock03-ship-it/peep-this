import { describe, expect, it } from "vitest";
import { fadeAt, FOLD_MS, FOLD_STEPS, insetFor, lerpBox, progressAt, REGION_LEAD, regionSteps, type Box } from "./fold";

const STRIP: Box = { x: 292, y: 0, w: 68, h: 170 };
const PANEL: Box = { x: 0, y: 0, w: 360, h: 420 };

describe("fold timing", () => {
  it("is the owner's 12 steps over 240ms", () => {
    expect([FOLD_STEPS, FOLD_MS]).toEqual([12, 240]);
  });

  it("moves on the click, steps every 20ms and ends exactly on 1", () => {
    expect(progressAt(0)).toBeCloseTo(1 / 12);
    expect(progressAt(19)).toBeCloseTo(1 / 12);
    expect(progressAt(20)).toBeCloseTo(2 / 12);
    expect(progressAt(239)).toBe(1);
    expect(progressAt(5000)).toBe(1);
    expect(progressAt(-5)).toBeCloseTo(1 / 12);
  });

  it("cross-fades in quarters", () => {
    expect(fadeAt(0)).toEqual({ out: 1, in: 0 });
    expect(fadeAt(0.25)).toEqual({ out: 0.5, in: 0 });
    expect(fadeAt(0.45)).toEqual({ out: 0, in: 0.5 });
    expect(fadeAt(1)).toEqual({ out: 0, in: 1 });
  });
});

describe("fold geometry", () => {
  it("the anchored corner stays put at every step", () => {
    for (let i = 0; i <= FOLD_STEPS; i++) {
      const b = lerpBox(STRIP, PANEL, i / FOLD_STEPS);
      expect(b.x + b.w).toBeCloseTo(360);
      expect(b.y).toBe(0);
    }
  });

  it("the hit region always covers the card, runs ahead of a growing one and trails a shrinking one", () => {
    const covers = (r: Box, b: Box) => r.x <= b.x + 1e-9 && r.y <= b.y + 1e-9 && r.x + r.w >= b.x + b.w - 1e-9 && r.y + r.h >= b.y + b.h - 1e-9;
    for (const [from, to] of [[STRIP, PANEL], [PANEL, STRIP]]) {
      const steps = regionSteps(from, to);
      expect(steps).toHaveLength(FOLD_STEPS + 1);
      // During interval i the card shows the end of that interval.
      steps.forEach((r, i) => expect(covers(r, lerpBox(from, to, progressAt(i * (FOLD_MS / FOLD_STEPS))))).toBe(true));
      expect(steps[steps.length - 1]).toEqual(to);
    }
    expect(regionSteps(STRIP, PANEL)[0]).toEqual(lerpBox(STRIP, PANEL, REGION_LEAD / FOLD_STEPS));
    expect(regionSteps(PANEL, STRIP)[0]).toEqual(PANEL);
  });

  it("insets clip an element to the frame and go negative where the frame is larger", () => {
    const el: Box = { x: 0, y: 0, w: 354, h: 414 };
    expect(insetFor(el, { x: 294, y: 2, w: 58, h: 160 })).toBe("inset(2.00px 2.00px 252.00px 294.00px)");
    expect(insetFor({ x: 292, y: 0, w: 62, h: 164 }, { x: 2, y: 2, w: 350, h: 410 })).toBe("inset(2.00px 2.00px -248.00px -290.00px)");
  });
});
