import { describe, expect, it } from "vitest";
import { bannerMode } from "./banner";

describe("bannerMode", () => {
  it("offers install or update until dismissed for that status", () => {
    expect(bannerMode("missing", null, false)).toBe("install");
    expect(bannerMode("outdated", null, false)).toBe("update");
    expect(bannerMode("installed", null, false)).toBeNull();
    expect(bannerMode("missing", "missing", false)).toBeNull();
    expect(bannerMode("outdated", "outdated", false)).toBeNull();
  });

  it("shows again when the status changes after a dismissal", () => {
    expect(bannerMode("outdated", "missing", false)).toBe("update");
  });

  it("opens on request regardless of dismissal", () => {
    expect(bannerMode("installed", null, true)).toBe("disconnect");
    expect(bannerMode("missing", "missing", true)).toBe("install");
    expect(bannerMode("outdated", "outdated", true)).toBe("update");
  });
});
