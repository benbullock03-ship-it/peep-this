import { describe, expect, it } from "vitest";
import { demoSnapshot, demoStep } from "./demo";
import { ageText, limitsTitle, limitWarning, pct, pressure, toLimitsView } from "./limits";

describe("limitsTitle", () => {
  it("carries both percentages and the age", () => {
    expect(limitsTitle({ fiveHour: 42, sevenDay: 7, at: 0 }, 60_000, 180_000)).toBe("5h 42% · 7d 7% · updated 2m ago");
  });
  it("adds a reset time only when it is known and still ahead", () => {
    const resetsAt = new Date(2026, 10, 13, 14, 20).getTime();
    const t = limitsTitle({ fiveHour: 82, sevenDay: 7, at: 0, fiveHourResetsAt: resetsAt, sevenDayResetsAt: 1 }, 0, resetsAt - 1);
    expect(t.startsWith("5h 82% (resets 14:20) · 7d 7% · ")).toBe(true);
  });
});

describe("pressure", () => {
  const v = (fiveHour: number, sevenDay: number) => ({ fiveHour, sevenDay, at: 0 });
  it("steps at 80, 95 and 100 on the higher limit", () => {
    expect(pressure(null)).toBe(0);
    expect(pressure(v(79, 10))).toBe(0);
    expect(pressure(v(10, 80))).toBe(80);
    expect(pressure(v(94, 10))).toBe(80);
    expect(pressure(v(95, 10))).toBe(95);
    expect(pressure(v(99, 100))).toBe(100);
  });
});

describe("limitWarning", () => {
  const rl = (fiveHourPct: number, sevenDayPct: number) => ({ fiveHourPct, sevenDayPct, at: 0 });
  it("names the higher limit from 80%, 5h on a tie", () => {
    expect(limitWarning(undefined)).toBeNull();
    expect(limitWarning(rl(79.4, 20))).toBeNull();
    expect(limitWarning(rl(81.6, 20))).toEqual({ key: "5h", pct: 82 });
    expect(limitWarning(rl(81, 90))).toEqual({ key: "7d", pct: 90 });
    expect(limitWarning(rl(85, 85))).toEqual({ key: "5h", pct: 85 });
  });
});

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
