import { describe, expect, it } from "vitest";
import { detailFields, formatDuration, truncateMiddle } from "./details";
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

describe("detailFields", () => {
  it("fills every field and dashes the missing ones", () => {
    const f = Object.fromEntries(detailFields(base, 6_000).map((d) => [d.key, d.value]));
    expect(f).toEqual({
      cwd: "F:\\Projects\\agentview widget",
      model: "—",
      tool: "—",
      since: "5s",
      ctx: "—",
      kind: "interactive · cli",
    });
  });
  it("joins tool and target, truncating the middle but keeping the full text", () => {
    const target = "src/some/really/long/directory/name/component.test.ts";
    const s = { ...base, model: "claude-opus-5-5", contextPct: 41.6, lastTool: { name: "Edit", target } };
    const tool = detailFields(s, 0).find((d) => d.key === "tool")!;
    expect(tool.full).toBe(`Edit ${target}`);
    expect(tool.value.startsWith("Edit src/")).toBe(true);
    expect(tool.value.endsWith("component.test.ts")).toBe(true);
    expect(detailFields(s, 0).find((d) => d.key === "ctx")!.value).toBe("42% used");
  });
});
