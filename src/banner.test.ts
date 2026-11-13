import { describe, expect, it } from "vitest";
import { bannerMode } from "./banner";

describe("bannerMode", () => {
  it("offers install or update until dismissed for that status", () => {
    expect(bannerMode("missing", null)).toBe("install");
    expect(bannerMode("outdated", null)).toBe("update");
    expect(bannerMode("installed", null)).toBeNull();
    expect(bannerMode("missing", "missing")).toBeNull();
    expect(bannerMode("outdated", "outdated")).toBeNull();
  });

  it("shows again when the status changes after a dismissal", () => {
    expect(bannerMode("outdated", "missing")).toBe("update");
  });
});
