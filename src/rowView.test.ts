import { describe, expect, it } from "vitest";
import { demoDecide, demoIdentity, demoSnapshot, demoStep } from "./demo";
import type { Session } from "./model";
import { contextFill, toRowView } from "./rowView";

const base: Session = {
  id: "abc12345-0000",
  pid: 1,
  name: "agentview",
  cwd: "F:\\Projects\\agentview widget",
  kind: "interactive",
  entrypoint: "cli",
  state: "working",
  stateSince: 0,
  task: "Edit src/main.ts",
  subagents: 0,
  alive: true,
};

describe("contextFill", () => {
  it("is null when absent and clamps to 0..100", () => {
    expect(contextFill(undefined)).toBeNull();
    expect(contextFill(Number.NaN)).toBeNull();
    expect(contextFill(-5)).toBe(0);
    expect(contextFill(42.5)).toBe(42.5);
    expect(contextFill(130)).toBe(100);
  });
});

describe("toRowView", () => {
  it("takes LED and label from the design and identity from the cwd", () => {
    const v = toRowView(base);
    expect(v).toMatchObject({ label: "Working", led: "#7FE6FF", color: "blue", accessory: "none", contextPct: null });
    expect(toRowView({ ...base, state: "waiting" })).toMatchObject({ label: "Needs you", led: "#FFB23E" });
  });

  it("falls back to the label when the task is empty and to the id when unnamed", () => {
    const v = toRowView({ ...base, state: "thinking", task: "", name: "" });
    expect(v.task).toBe("Thinking");
    expect(v.name).toBe("abc12345");
  });
});

describe("demo", () => {
  it("renders the six design agents with their design identities", () => {
    const snap = demoSnapshot(1_000_000);
    expect(snap.sessions.map((x) => x.name)).toEqual(["ingest-01", "research-02", "triage-03", "digest-04", "sync-05", "scrape-06"]);
    expect(toRowView(snap.sessions[2], demoIdentity)).toMatchObject({ color: "pink", accessory: "headphones", task: "Waiting for approval" });
  });

  it("changes exactly one agent per step", () => {
    const a = demoSnapshot(0);
    const b = demoStep(a, 5);
    expect(a.sessions.filter((x, i) => x.state !== b.sessions[i].state)).toHaveLength(1);
  });

  it("holds one card of each tier on waiting rows, and a decision clears it", () => {
    const a = demoSnapshot(0);
    const held = a.sessions.filter((s) => s.pending?.length);
    expect(held.map((s) => s.pending![0].risk.tier).sort()).toEqual(["destructive", "safe", "shell"]);
    expect(held.every((s) => s.state === "waiting")).toBe(true);
    let s = a;
    for (let t = 1; t < 50; t++) s = demoStep(s, t);
    expect(s.sessions.filter((x) => x.pending?.length)).toHaveLength(3);
    const b = demoDecide(a, held[0].pending![0].id, "deny", 9);
    expect(b.sessions.find((s) => s.id === held[0].id)).toMatchObject({ state: "thinking", pending: undefined });
  });
});
