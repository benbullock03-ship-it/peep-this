import { describe, expect, it } from "vitest";
import { formatDuration, openFacts, truncateMiddle } from "./details";
import type { Session } from "./model";

const base: Session = {
  id: "abc",
  pid: 1,
  name: "agentview",
  cwd: "F:\\Projects\\agentview widget",
  kind: "interactive",
  entrypoint: "cli",
  state: "working",
  stateSince: 1_000,
  task: "Edit",
  subagents: 0,
  alive: true,
};

describe("truncateMiddle", () => {
  it("leaves short strings alone", () => {
    expect(truncateMiddle("abc", 5)).toBe("abc");
    expect(truncateMiddle("abcde", 5)).toBe("abcde");
  });
  it("keeps both ends at exactly max chars", () => {
    const out = truncateMiddle("F:/Projects/some/very/deep/path/file.ts", 15);
    expect(out).toBe("F:/Proj…file.ts");
    expect(Array.from(out)).toHaveLength(15);
  });
});

describe("formatDuration", () => {
  it("scales units", () => {
    expect(formatDuration(-5)).toBe("0s");
    expect(formatDuration(42_900)).toBe("42s");
    expect(formatDuration(243_000)).toBe("4m 03s");
    expect(formatDuration(3_725_000)).toBe("1h 02m");
  });
});

describe("openFacts", () => {
  it("fills every fact and leaves the missing ones empty", () => {
    expect(openFacts(base, 6_000)).toEqual({
      since: "Working for 5s",
      model: "",
      kind: "interactive · cli",
      tool: "—",
      toolFull: "",
      ctx: null,
      cwd: "F:\\Projects\\agentview widget",
    });
  });
  it("joins tool and target, truncating the middle but keeping the full text", () => {
    const target = "src/some/really/long/directory/name/component.test.ts";
    const f = openFacts({ ...base, state: "waiting", model: "Opus 5.5", contextPct: 141.6, lastTool: { name: "Edit", target } }, 1_000);
    expect(f.toolFull).toBe(`Edit ${target}`);
    expect(f.tool.startsWith("Edit src/")).toBe(true);
    expect(f.tool.endsWith(".test.ts")).toBe(true);
    expect(f.ctx).toBe(100);
    expect(f.since).toBe("Needs you for 0s");
  });
});
