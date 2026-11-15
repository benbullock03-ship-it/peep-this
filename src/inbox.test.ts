import { describe, expect, it } from "vitest";
import { badgeText, comboFromKey, inboxOrder, step, successor, type InboxItem } from "./inbox";
import type { PendingApproval, Session } from "./model";

const req = (id: string, receivedAt: number, agentId?: string): PendingApproval => ({
  id,
  tool: "Bash",
  input: { command: "ls" },
  cwd: "C:/w",
  agentId,
  receivedAt,
  releaseAt: receivedAt + 60_000,
  risk: { tier: "shell", reason: "" },
});

const session = (id: string, pending: PendingApproval[], alive = true): Session => ({
  id,
  pid: 1,
  name: id,
  cwd: "C:/w",
  kind: "interactive",
  entrypoint: "cli",
  state: "waiting",
  stateSince: 0,
  task: "",
  subagents: 0,
  alive,
  pending,
});

const ids = (order: InboxItem[]) => order.map((x) => x.id);

describe("inboxOrder", () => {
  it("puts the oldest request first across sessions and their subagents", () => {
    const order = inboxOrder([
      session("a", [req("a1", 30), req("a2", 10, "agent-x")]),
      session("b", [req("b1", 20)]),
      session("c", [req("c1", 5)], false),
    ]);
    expect(ids(order)).toEqual(["a2", "b1", "a1"]);
    expect(order[0]).toEqual({ id: "a2", receivedAt: 10, agentId: "agent-x", sessionId: "a" });
  });

  it("queues questions after real approvals, however old", () => {
    const q: PendingApproval = { ...req("q1", 1), tool: "AskUserQuestion", risk: { tier: "question", reason: "" } };
    expect(ids(inboxOrder([session("a", [q, req("a1", 30)]), session("b", [req("b1", 20)])]))).toEqual(["b1", "a1", "q1"]);
  });

  it("breaks receipt ties by id and is empty with nothing pending", () => {
    expect(ids(inboxOrder([session("a", [req("ap-2", 1)]), session("b", [req("ap-1", 1)])]))).toEqual(["ap-1", "ap-2"]);
    expect(inboxOrder([session("a", [])])).toEqual([]);
  });
});

const items = (...xs: string[]): InboxItem[] => xs.map((id, i) => ({ id, receivedAt: i, sessionId: "s" }));

describe("step", () => {
  it("moves forward and back with wrap-around", () => {
    const o = items("x", "y", "z");
    expect(step(o, "x", 1)).toBe("y");
    expect(step(o, "z", 1)).toBe("x");
    expect(step(o, "x", -1)).toBe("z");
    expect(step(o, "y", -1)).toBe("x");
  });

  it("starts from the matching end when current is gone, and is null when empty", () => {
    const o = items("x", "y");
    expect(step(o, "gone", 1)).toBe("x");
    expect(step(o, null, -1)).toBe("y");
    expect(step([], "x", 1)).toBeNull();
  });
});

describe("successor", () => {
  it("keeps the current card while it is still pending, even if newer ones arrive", () => {
    expect(successor(items("x", "y"), items("x", "y", "z"), "y")).toBe("y");
  });

  it("moves to the next request after a decision", () => {
    expect(successor(items("x", "y", "z"), items("x", "z"), "y")).toBe("z");
  });

  it("skips requests that went at the same time and wraps to the oldest", () => {
    expect(successor(items("x", "y", "z"), items("w", "x"), "y")).toBe("w");
    expect(successor(items("x", "y"), items("x"), "y")).toBe("x");
  });

  it("starts from the oldest without a current card", () => {
    expect(successor([], items("x", "y"), null)).toBe("x");
  });

  it("is null once the inbox is empty, which returns focus", () => {
    expect(successor(items("x"), [], "x")).toBeNull();
  });
});

describe("badgeText", () => {
  it("caps at 9+", () => {
    expect([1, 9, 10, 42].map(badgeText)).toEqual(["1", "9", "9+", "9+"]);
  });
});

describe("comboFromKey", () => {
  const k = (code: string, mods: Partial<Record<"ctrlKey" | "altKey" | "shiftKey" | "metaKey", boolean>> = {}) =>
    comboFromKey({ code, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...mods });

  it("builds the backend's canonical form", () => {
    expect(k("KeyI", { ctrlKey: true, altKey: true })).toBe("Ctrl+Alt+I");
    expect(k("Digit5", { metaKey: true, shiftKey: true })).toBe("Shift+Win+5");
    expect(k("F12", { ctrlKey: true })).toBe("Ctrl+F12");
    expect(k("ArrowUp", { altKey: true })).toBe("Alt+Up");
  });

  it("waits while only modifiers are down and ignores unsupported keys", () => {
    expect(k("ControlLeft", { ctrlKey: true })).toBeNull();
    expect(k("F25", { ctrlKey: true })).toBeNull();
    expect(k("Semicolon", { ctrlKey: true })).toBeNull();
  });
});
