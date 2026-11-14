import { describe, expect, it } from "vitest";
import type { TrailEntry } from "./model";
import { entryTitle, filterTrail, formatWhen, markOf, sessionOptions, shownTarget, trailKey } from "./trail";

const NOW = new Date(2026, 10, 14, 15, 30, 0).getTime();

function e(over: Partial<TrailEntry>): TrailEntry {
  return {
    t: new Date(2026, 10, 14, 9, 5, 3).getTime(),
    sessionId: "s1",
    session: "widget",
    cwd: "C:\\work\\widget",
    tool: "Bash",
    target: "npm test",
    tier: "shell",
    outcome: "allowed",
    ...over,
  };
}

describe("formatWhen", () => {
  it("shows seconds today and the date otherwise", () => {
    expect(formatWhen(new Date(2026, 10, 14, 9, 5, 3).getTime(), NOW)).toBe("09:05:03");
    expect(formatWhen(new Date(2026, 10, 3, 14, 5, 59).getTime(), NOW)).toBe("3 Nov 14:05");
    expect(formatWhen(new Date(2025, 10, 14, 9, 0).getTime(), NOW)).toBe("14 Nov 09:00");
  });
});

describe("marks and titles", () => {
  it("ticks allows, crosses denies, dashes releases", () => {
    expect([markOf("allowed"), markOf("auto-allowed"), markOf("denied"), markOf("released")]).toEqual(["✓", "✓", "✗", "–"]);
  });

  it("puts time, outcome, release cause and reason in the title", () => {
    expect(entryTitle(e({ outcome: "denied", reason: "not on main" }), NOW)).toBe(
      "09:05:03 · denied\nReason: not on main\nBash npm test",
    );
    expect(entryTitle(e({ outcome: "released", via: "timed-out" }), NOW)).toBe(
      "09:05:03 · released to terminal (timed out)\nBash npm test",
    );
    expect(entryTitle(e({ outcome: "auto-allowed", rule: "r-3", target: "" }), NOW)).toBe("09:05:03 · auto-allowed · rule r-3\nBash");
  });

  it("shows paths inside the cwd relative", () => {
    expect(shownTarget(e({ tool: "Edit", target: "C:\\work\\widget\\src\\a.ts" }))).toBe("src/a.ts");
    expect(shownTarget(e({ target: "rm -rf dist" }))).toBe("rm -rf dist");
  });
});

describe("filterTrail", () => {
  const log = [
    e({ t: 3, sessionId: "s1", outcome: "allowed", target: "npm test" }),
    e({ t: 2, sessionId: "s2", session: "api", outcome: "denied", reason: "Wrong Branch", target: "git push" }),
    e({ t: 1, sessionId: "s1", outcome: "released", via: "timed-out", tool: "Write", target: "C:\\work\\widget\\x.md" }),
  ];

  it("filters by session, outcome and text together", () => {
    const none = { session: "", outcome: "" as const, text: "" };
    expect(filterTrail(log, none)).toHaveLength(3);
    expect(filterTrail(log, { ...none, session: "s1" }).map((x) => x.t)).toEqual([3, 1]);
    expect(filterTrail(log, { ...none, outcome: "denied" }).map((x) => x.t)).toEqual([2]);
    expect(filterTrail(log, { ...none, text: "wrong branch" }).map((x) => x.t)).toEqual([2]);
    expect(filterTrail(log, { ...none, text: " WRITE " }).map((x) => x.t)).toEqual([1]);
    expect(filterTrail(log, { session: "s1", outcome: "denied", text: "" })).toEqual([]);
  });

  it("lists sessions newest first under their latest name", () => {
    const opts = sessionOptions([...log, e({ t: 9, sessionId: "s2", session: "api-renamed" })]);
    expect(opts).toEqual([
      { id: "s2", name: "api-renamed" },
      { id: "s1", name: "widget" },
    ]);
  });

  it("keys change only when marks would", () => {
    expect(trailKey(undefined)).toBe("");
    expect(trailKey(log)).toBe(trailKey([...log]));
    expect(trailKey(log)).not.toBe(trailKey(log.slice(1)));
  });
});
