import { describe, expect, it } from "vitest";
import { arcCounts, arcTip } from "./arc";

describe("arcCounts", () => {
  it("leads with closed and drops zero counts", () => {
    expect(arcCounts({ closed: 41, open: 12, parked: 3 }).map((c) => `${c.n} ${c.label}`)).toEqual(["41 closed", "12 open", "3 parked"]);
    expect(arcCounts({ closed: 41, open: 12, parked: 3 })[0].lead).toBe(true);
    expect(arcCounts({ closed: 6, open: 9, parked: 0 }).map((c) => c.label)).toEqual(["closed", "open"]);
    expect(arcCounts({ closed: 0, open: 4, parked: 0 }).map((c) => c.label)).toEqual(["open"]);
  });
});

describe("arcTip", () => {
  it("shows the full north star, every count and the last ticket's title", () => {
    const arc = { closed: 1, open: 0, parked: 3, last: { id: "AV-70", title: "cheer" } };
    expect(arcTip({ northStar: "A calm desk pet", arc })).toBe("A calm desk pet\n1 closed · 3 parked\nLast closed: AV-70 · cheer");
    expect(arcTip({ arc: { ...arc, last: { id: "AV-70", title: "" } } })).toBe("No north star yet. Walk Through pins one down.\n1 closed · 3 parked\nLast closed: AV-70");
    expect(arcTip({ northStar: "x", arc: { closed: 0, open: 0, parked: 0 } })).toBe("x\nNone closed yet");
  });
});
