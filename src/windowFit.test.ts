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

  it("ignores zero and non-finite heights", () => {
    const seen: number[] = [];
    const report = createHeightReporter((h) => seen.push(h));
    report(0);
    report(Number.NaN);
    report(Infinity);
    expect(seen).toEqual([]);
  });
});
