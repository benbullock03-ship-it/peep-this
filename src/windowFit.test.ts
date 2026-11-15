import { describe, expect, it } from "vitest";
import { createHeightReporter } from "./windowFit";

describe("createHeightReporter", () => {
  it("rounds up and skips repeats", () => {
    const seen: number[] = [];
    const report = createHeightReporter((h) => seen.push(h));
    report(120.2);
    report(120.9);
    report(121);
    report(200);
    expect(seen).toEqual([121, 200]);
  });

  it("reports the same length again under a new tag", () => {
    const seen: [number, string][] = [];
    const report = createHeightReporter((h, tag) => seen.push([h, tag]));
    report(300, "column");
    report(300, "column");
    report(300, "row");
    report(300, "column");
    expect(seen).toEqual([[300, "column"], [300, "row"], [300, "column"]]);
  });

  it("ignores zero and non-finite heights", () => {
    const seen: number[] = [];
    const report = createHeightReporter((h) => seen.push(h));
    report(0);
    report(Number.NaN);
    report(Infinity);
    expect(seen).toEqual([]);
  });
});
