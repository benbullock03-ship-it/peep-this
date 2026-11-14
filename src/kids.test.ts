import { describe, expect, it } from "vitest";
import { identityFor } from "./identity";
import { KID_LINGER_MS, KID_MAX_RUNNING, kidAccessory, kidView, nextFoldAt, planKids } from "./kids";
import type { Agent, PendingApproval } from "./model";
import { toRowView } from "./rowView";

const T = 50_000_000;

const agent = (id: string, over: Partial<Agent> = {}): Agent => ({
  id,
  agentType: "Explore",
  description: "",
  state: "working",
  stateSince: T - 1000,
  task: "Read a.ts",
  lastTool: { name: "Read", target: "a.ts" },
  startedAt: T - 10_000,
  toolCount: 1,
  ...over,
});

const done = (id: string, endedAt: number, over: Partial<Agent> = {}) =>
  agent(id, { state: "success", task: "Done", endedAt, startedAt: endedAt - 64_000, toolCount: 3, result: "Found it", ...over });

describe("kidAccessory", () => {
  it("depends on the agent type only, never on the project", () => {
    expect(kidAccessory("Explore")).toBe(kidAccessory("explore "));
    const types = ["Explore", "Plan", "general-purpose", "code-reviewer", "statusline-setup", "claude-code-guide"];
    const accs = new Set(types.map(kidAccessory));
    expect(accs.size).toBeGreaterThan(1);
    expect(accs.has("none")).toBe(false);
    expect(kidAccessory("")).toBe(kidAccessory("agent"));
  });

  it("gives the same child the same look under two projects, in each parent's colour", () => {
    const a = agent("x", { agentType: "Plan" });
    const v = kidView(a, undefined, T);
    expect(kidView({ ...a, id: "y" }, undefined, T).accessory).toBe(v.accessory);
    const base = { pid: 1, name: "p", kind: "interactive", entrypoint: "cli", state: "idle" as const, stateSince: T, task: "", subagents: 0, alive: true };
    const p1 = toRowView({ ...base, id: "1", cwd: "C:/one" });
    const p2 = toRowView({ ...base, id: "2", cwd: "C:/projects/two" });
    expect(p1.color).toBe(identityFor("C:/one").color);
    expect(p2.color).toBe(identityFor("C:/projects/two").color);
  });
});

describe("fold timing", () => {
  it("keeps a finished child as a row for 30s, then counts it in done N", () => {
    const end = T - 1000;
    const kids = [agent("run"), done("fin", end)];
    expect(planKids(kids, end + KID_LINGER_MS - 1).shown.map((a) => a.id)).toEqual(["run", "fin"]);
    const later = planKids(kids, end + KID_LINGER_MS);
    expect(later.shown.map((a) => a.id)).toEqual(["run"]);
    expect(later.done).toBe(1);
  });

  it("shows folded children again, newest first, when the counter is opened", () => {
    const kids = [done("old", T - 200_000), done("new", T - 100_000)];
    expect(planKids(kids, T, { showDone: true }).shown.map((a) => a.id)).toEqual(["new", "old"]);
  });

  it("schedules the next fold at the earliest lingering child", () => {
    expect(nextFoldAt([agent("run"), done("a", T - 5000), done("b", T - 2000), done("old", T - 99_000)], T)).toBe(T - 5000 + KID_LINGER_MS);
    expect(nextFoldAt([agent("run")], T)).toBeNull();
  });

  it("states result, duration and tool count while lingering", () => {
    expect(kidView(done("f", T - 1000), undefined, T)).toMatchObject({ finished: true, state: "success", say: "Found it", stats: "1m 04s · 3 tools" });
    expect(kidView(done("f", T, { toolCount: 1, result: undefined }), undefined, T)).toMatchObject({ say: "All done!", stats: "1m 04s · 1 tool" });
    expect(kidView(done("f", T, { state: "idle", result: undefined }), undefined, T)).toMatchObject({ label: "Stopped" });
  });
});

describe("overflow", () => {
  const running = (n: number) => Array.from({ length: n }, (_, i) => agent(`a${i}`, { startedAt: T - 100_000 + i }));

  it("shows at most 4 running and counts the rest as +N more", () => {
    const plan = planKids(running(7), T);
    expect(plan.shown.map((a) => a.id)).toEqual(["a0", "a1", "a2", "a3"]);
    expect(KID_MAX_RUNNING).toBe(4);
    expect(plan.more).toBe(3);
  });

  it("4 or fewer have no more row; expanding shows them all", () => {
    expect(planKids(running(4), T)).toMatchObject({ more: 0, overflow: 0 });
    expect(planKids(running(6), T, { allRunning: true })).toMatchObject({ more: 0, overflow: 2 });
    expect(planKids(running(6), T, { allRunning: true }).shown).toHaveLength(6);
  });

  it("a child holding an approval is never hidden behind +N more", () => {
    const ask: PendingApproval = { id: "r1", tool: "Bash", input: { command: "ls" }, cwd: "C:/p", agentId: "a5", receivedAt: T, releaseAt: T + 60_000, risk: { tier: "shell", reason: "shell" } };
    const plan = planKids(running(6), T, { pending: [ask] });
    expect(plan.shown[0].id).toBe("a5");
    expect(kidView(plan.shown[0], [ask], T)).toMatchObject({ state: "waiting", attention: true, say: "Can I run ls?" });
  });
});
