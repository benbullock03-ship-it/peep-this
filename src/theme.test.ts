import { describe, expect, it } from "vitest";
import { parsePref, resolveTheme } from "./theme";

describe("theme", () => {
  it("system follows the OS, explicit choices ignore it", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
    expect(resolveTheme("light", true)).toBe("light");
  });

  it("unknown or missing stored values fall back to system", () => {
    expect(parsePref(null)).toBe("system");
    expect(parsePref("sepia")).toBe("system");
    expect(parsePref("light")).toBe("light");
  });
});
