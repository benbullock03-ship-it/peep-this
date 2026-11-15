import { describe, expect, it } from "vitest";
import { KEEPS_PEEK, pinsPeek } from "./peek";

/** A target inside elements matching `within`, as `closest` would see it. */
const at = (...within: string[]) => ({
  closest: (sel: string) => (sel.split(",").some((s) => within.includes(s.trim())) ? {} : null),
});

describe("pinsPeek", () => {
  it("pins on a press anywhere that is not a row action", () => {
    expect(pinsPeek(at())).toBe(true);
    expect(pinsPeek(at(".ps-row"))).toBe(true);
    expect(pinsPeek(at(".card-reason-toggle"))).toBe(true);
  });

  it("keeps the peek for focusing a terminal, allow, deny and collapse", () => {
    for (const s of [".ps-item .ps-peep", ".card-allow", ".card-deny", "#collapse-btn"]) {
      expect(KEEPS_PEEK).toContain(s);
      expect(pinsPeek(at(s))).toBe(false);
    }
  });

  it("ignores a press with no target", () => {
    expect(pinsPeek(null)).toBe(false);
  });
});
