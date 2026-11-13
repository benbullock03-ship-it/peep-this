import { describe, expect, it } from "vitest";
import { demoSnapshot, demoStep } from "./demo";
import { ageText, pct, toLimitsView } from "./limits";

describe("pct", () => {
  it("rounds, clamps and rejects non-numbers", () => {
    expect(pct(42.4)).toBe(42);
    expect(pct(42.5)).toBe(43);
    expect(pct(-3)).toBe(0);
    expect(pct(120)).toBe(100);
    expect(pct(undefined)).toBeNull();
    expect(pct(null)).toBeNull();
    expect(pct(Number.NaN)).toBeNull();
    expect(pct(Number.POSITIVE_INFINITY)).toBeNull();
  });
});

describe("ageText", () => {
  const now = 10_000_000;
  it("picks the largest whole unit", () => {
    expect(ageText(now, now)).toBe("updated just now");
    expect(ageText(now - 4_999, now)).toBe("updated just now");
    expect(ageText(now - 12_000, now)).toBe("updated 12s ago");
    expect(ageText(now - 59_999, now)).toBe("updated 59s ago");
    expect(ageText(now - 60_000, now)).toBe("updated 1m ago");
    expect(ageText(now - 3_599_000, now)).toBe("updated 59m ago");
    expect(ageText(now - 7_200_000, now)).toBe("updated 2h ago");
    expect(ageText(now - 3 * 86_400_000, now)).toBe("updated 3d ago");
  });
  it("treats a future timestamp as fresh", () => {
    expect(ageText(now + 5_000, now)).toBe("updated just now");
  });
});

describe("toLimitsView", () => {
  it("is null until rate limits arrive", () => {
    expect(toLimitsView(undefined)).toBeNull();
    expect(toLimitsView({ fiveHourPct: Number.NaN, sevenDayPct: 1, at: 0 })).toBeNull();
  });
  it("rounds both values and keeps the payload time", () => {
    expect(toLimitsView({ fiveHourPct: 41.6, sevenDayPct: 18.2, at: 5 })).toEqual({ fiveHour: 42, sevenDay: 18, at: 5 });
  });
});

describe("demo rate limits", () => {
  it("are present and stay fresh across steps", () => {
    const snap = demoSnapshot(1_000);
    expect(snap.rateLimits).toEqual({ fiveHourPct: 42, sevenDayPct: 18, at: 1_000 });
    expect(demoStep(snap, 5_000, () => 0.5).rateLimits?.at).toBe(5_000);
  });
});
