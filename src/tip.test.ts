import { describe, expect, it } from "vitest";
import { STATES } from "./model";
import { tipFor } from "./tip";

describe("tipFor", () => {
  const s = { tip: "crowded" as const, alive: true };

  it("shows only while idle or done", () => {
    const shown = STATES.filter((st) => tipFor(s, st, false) !== null);
    expect(shown).toEqual(["idle", "success"]);
  });

  it("gives way to a nudge, a dismissal and an ended session", () => {
    expect(tipFor(s, "idle", true)).toBeNull();
    expect(tipFor(s, "idle", false, "crowded")).toBeNull();
    expect(tipFor(s, "idle", false, "helper")).toBe("crowded");
    expect(tipFor({ ...s, alive: false }, "idle", false)).toBeNull();
    expect(tipFor({ alive: true }, "idle", false)).toBeNull();
  });
});
