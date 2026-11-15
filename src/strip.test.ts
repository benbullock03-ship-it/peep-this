import { describe, expect, it } from "vitest";
import type { Session, State } from "./model";
import { sortSessions } from "./sort";
import { STRIP_MAX, stripView } from "./strip";

function s(id: string, state: State, stateSince: number, extra: Partial<Session> = {}): Session {
  return { id, pid: 1, name: id, cwd: `c:/p/${id}`, kind: "interactive", entrypoint: "cli", state, stateSince, task: "", subagents: 0, alive: true, ...extra };
}

const many = (n: number) => Array.from({ length: n }, (_, i) => s(`s${i}`, "working", 1000 - i));

describe("stripView", () => {
  it("has one cell per session in the panel's sort order, attention first", () => {
    const sessions = [s("a", "working", 100), s("b", "waiting", 10), s("c", "idle", 300), s("d", "error", 50)];
    const v = stripView(sessions);
    expect(v.cells.map((c) => c.id)).toEqual(sortSessions(sessions).map((x) => x.id));
    expect(v.cells.map((c) => c.id)).toEqual(["d", "b", "c", "a"]);
    expect(v.cells.map((c) => c.attention)).toEqual([true, true, false, false]);
    expect(v.more).toBe(0);
  });

  it("carries the session's real state, colour and accessory", () => {
    const [c] = stripView([s("a", "thinking", 1)], () => ({ color: "pink", accessory: "bow" })).cells;
    expect(c).toMatchObject({ state: "thinking", label: "Thinking", color: "pink", accessory: "bow", alive: true });
  });

  it("shows up to 8 peeps with no +N tile", () => {
    expect(stripView(many(STRIP_MAX))).toMatchObject({ more: 0, cells: { length: 8 } });
    expect(stripView([])).toEqual({ cells: [], more: 0 });
  });

  it("caps at 8 peeps and counts the rest", () => {
    const v = stripView(many(11));
    expect(v.cells.map((c) => c.id)).toEqual(["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7"]);
    expect(v.more).toBe(3);
  });

  it("keeps a waiting session in view when it would otherwise fall behind +N", () => {
    const v = stripView([...many(9), s("late", "waiting", 1)]);
    expect(v.cells[0].id).toBe("late");
    expect(v.more).toBe(2);
  });

  it("gives subagents no cell of their own", () => {
    const agent = { id: "k", agentType: "Explore", description: "", state: "working" as State, stateSince: 1, task: "", startedAt: 1, toolCount: 0 };
    expect(stripView([s("a", "working", 1, { subagents: 1, agents: [agent] })]).cells).toHaveLength(1);
  });

  it("flags a parent whose shell failed", () => {
    const sh = { command: "npm run build", description: "", stateSince: 1, startedAt: 1 };
    const shells = [
      { ...sh, id: "1", state: "running" as const },
      { ...sh, id: "2", state: "failed" as const, endedAt: 2, exitCode: 3, lastLine: "build broke" },
    ];
    const v = stripView([s("a", "working", 2, { shells }), s("b", "working", 1)]);
    expect(v.cells.map((c) => c.shellFailed)).toEqual([true, false]);
    expect(v.cells[0].shellNote).toBe("Shell failed: npm run build\nexit code 3\nlast: build broke");
  });
});
