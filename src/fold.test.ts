import { describe, expect, it } from "vitest";
import { bounds, fadeAt, FOLD_VARIANTS, insetFor, isFoldVariant, lerpBox, progressAt, REGION_LEAD, regionSteps, timeline, type Box } from "./fold";

const STRIP: Box = { x: 292, y: 0, w: 68, h: 170 };
const PANEL: Box = { x: 0, y: 0, w: 360, h: 420 };

describe("fold timelines", () => {
  it("a is 12 even steps over 240ms", () => {
    const t = timeline("a");
    expect(t.ms).toBe(240);
    expect(t.samples).toHaveLength(13);
    expect(t.samples[6]).toBeCloseTo(0.5);
    expect(t.stepped.every(Boolean)).toBe(true);
  });

  it("b glides for 220ms and ends on two hard steps", () => {
    const t = timeline("b");
    expect(t.ms).toBe(220);
    expect(t.stepped.slice(0, -2).some(Boolean)).toBe(false);
    expect(t.stepped.slice(-2)).toEqual([true, true]);
    expect(t.samples.slice(-3)).toEqual([0.94, 0.97, 1]);
    // Ease-out: more ground in the first interval than in the last of the glide.
    expect(t.samples[1] - t.samples[0]).toBeGreaterThan(t.samples[11] - t.samples[10]);
  });

  it("every timeline starts at 0, ends at 1, never goes back, and has one easing per interval", () => {
    for (const v of FOLD_VARIANTS) {
      const t = timeline(v);
      expect(t.samples[0]).toBe(0);
      expect(t.samples[t.samples.length - 1]).toBe(1);
      expect(t.samples.every((p, i) => i === 0 || p >= t.samples[i - 1])).toBe(true);
      expect(t.stepped).toHaveLength(t.samples.length - 1);
    }
  });

  it("a stepped timeline moves on the click and a smooth one glides", () => {
    const a = timeline("a");
    expect(progressAt(a, 0)).toBeCloseTo(1 / 12);
    expect(progressAt(a, 19)).toBeCloseTo(1 / 12);
    expect(progressAt(a, 20)).toBeCloseTo(2 / 12);
    expect(progressAt(a, 239)).toBe(1);
    const b = timeline("b");
    expect(progressAt(b, 0)).toBe(0);
    const gap = b.ms / 13;
    expect(progressAt(b, gap / 2)).toBeCloseTo(b.samples[1] / 2);
    expect(progressAt(b, gap * 11.5)).toBe(0.97);
    expect(progressAt(b, gap * 12.5)).toBe(1);
    expect(progressAt(b, 5000)).toBe(1);
  });

  it("the cross-fade steps in quarters when the timeline steps", () => {
    expect(fadeAt(0, true)).toEqual({ out: 1, in: 0 });
    expect(fadeAt(0.25, true)).toEqual({ out: 0.5, in: 0 });
    expect(fadeAt(0.45, false).in).toBeCloseTo(0.5);
    expect(fadeAt(1, true)).toEqual({ out: 0, in: 1 });
  });

  it("only a, b and c are timings", () => {
    expect(isFoldVariant("b")).toBe(true);
    expect(isFoldVariant("d")).toBe(false);
    expect(isFoldVariant(null)).toBe(false);
  });
});

describe("fold geometry", () => {
  it("the anchored corner stays put at every sample", () => {
    for (const p of timeline("b").samples) {
      const b = lerpBox(STRIP, PANEL, p);
      expect(b.x + b.w).toBeCloseTo(360);
      expect(b.y).toBe(0);
    }
  });

  it("the hit region always covers the card, runs ahead of a growing one and trails a shrinking one", () => {
    const t = timeline("a");
    const covers = (r: Box, b: Box) => r.x <= b.x + 1e-9 && r.y <= b.y + 1e-9 && r.x + r.w >= b.x + b.w - 1e-9 && r.y + r.h >= b.y + b.h - 1e-9;
    for (const [from, to] of [[STRIP, PANEL], [PANEL, STRIP]]) {
      const steps = regionSteps(t, from, to);
      expect(steps).toHaveLength(t.samples.length);
      steps.forEach((r, i) => expect(covers(r, lerpBox(from, to, t.samples[i]))).toBe(true));
      expect(steps[steps.length - 1]).toEqual(to);
    }
    const grow = regionSteps(t, STRIP, PANEL);
    expect(grow[0]).toEqual(lerpBox(STRIP, PANEL, t.samples[REGION_LEAD]));
    // A stepped card shows the end of its interval; the region is still ahead of it.
    expect(covers(grow[0], lerpBox(STRIP, PANEL, progressAt(t, 0)))).toBe(true);
    expect(regionSteps(t, PANEL, STRIP)[0]).toEqual(PANEL);
  });

  it("the zoom-rect region keeps the resting strip clickable", () => {
    const row: Box = { x: 0, y: 0, w: 480, h: 68 };
    const steps = regionSteps(timeline("c"), row, PANEL, [row]);
    expect(steps[steps.length - 1]).toEqual(bounds(row, PANEL));
  });

  it("insets clip an element to the frame and go negative where the frame is larger", () => {
    const el: Box = { x: 0, y: 0, w: 354, h: 414 };
    expect(insetFor(el, { x: 294, y: 2, w: 58, h: 160 })).toBe("inset(2.00px 2.00px 252.00px 294.00px)");
    expect(insetFor({ x: 292, y: 0, w: 62, h: 164 }, { x: 2, y: 2, w: 350, h: 410 })).toBe("inset(2.00px 2.00px -248.00px -290.00px)");
  });
});
