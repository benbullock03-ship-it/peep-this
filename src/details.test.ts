import { describe, expect, it } from "vitest";
import { formatDuration, openFacts } from "./details";
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
      branch: "",
      tool: "—",
      ctx: null,
      cwd: "F:\\Projects\\agentview widget",
    });
  });
  it("joins tool and target in full, never cutting the middle", () => {
    const target = ".venv/Scripts/python - <<'EOF'";
    const f = openFacts({ ...base, state: "waiting", model: "Opus 5.5", contextPct: 141.6, lastTool: { name: "Bash", target } }, 1_000);
    expect(f.tool).toBe(`Bash ${target}`);
    expect(f.ctx).toBe(100);
    expect(f.since).toBe("Needs you for 0s");
    expect(openFacts({ ...base, branch: "feature/login" }, 1_000).branch).toBe("feature/login");
  });
});
