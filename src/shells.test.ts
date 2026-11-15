import { describe, expect, it } from "vitest";
import { KID_LINGER_MS } from "./kids";
import type { Session, Shell } from "./model";
import { toRowView } from "./rowView";
import {
  charsThatFit,
  failNote,
  middleTruncate,
  nextShellFoldAt,
  planShells,
  SHELL_LINGER_MS,
  shellLabel,
  shellRuntime,
  shellTitle,
} from "./shells";

const T = 80_000_000;

const shell = (id: string, over: Partial<Shell> = {}): Shell => ({
  id,
  command: "npm run dev",
  description: "Start the dev server",
  state: "running",
  stateSince: T - 5000,
  startedAt: T - 64_000,
  ...over,
});

const ended = (id: string, endedAt: number, over: Partial<Shell> = {}) => shell(id, { state: "exited", exitCode: 0, endedAt, stateSince: endedAt, ...over });

describe("middleTruncate", () => {
  it("leaves short commands alone and keeps both ends of long ones", () => {
    expect(middleTruncate("npm run dev", 20)).toBe("npm run dev");
    expect(middleTruncate("npm run dev -- --port 5173", 15)).toBe("npm run…rt 5173");
    expect(middleTruncate("npm run dev -- --port 5173", 15)).toHaveLength(15);
  });

  it("degrades to an ellipsis when nothing fits", () => {
    expect(middleTruncate("abcdef", 1)).toBe("…");
    expect(middleTruncate("abcdef", 0)).toBe("…");
    expect(middleTruncate("abcdef", 2)).toBe("a…");
  });

  it("counts code points, never splitting a surrogate pair", () => {
    expect(middleTruncate("echo 🚀🚀🚀🚀 done", 9)).toBe("echo…done");
  });

  it("fits monospace characters to a width", () => {
    expect(charsThatFit(132, 11)).toBe(20);
    expect(charsThatFit(0, 11)).toBe(0);
  });
});

describe("chip text", () => {
  it("runs live while running and freezes at the end", () => {
    expect(shellRuntime(shell("a"), T)).toBe("1m 04s");
    expect(shellRuntime(shell("a"), T + 1000)).toBe("1m 05s");
    expect(shellRuntime(ended("b", T - 4000), T + 99_000)).toBe("1m 00s");
  });

  it("shows the port when serving and the exit code when failed", () => {
    expect(shellLabel(shell("a", { state: "serving", port: 5173 }))).toBe("Serving :5173");
    expect(shellLabel(shell("a", { state: "serving" }))).toBe("Serving");
    expect(shellLabel(shell("a", { state: "failed", exitCode: 3 }))).toBe("Failed 3");
    expect(shellLabel(shell("a", { state: "failed" }))).toBe("Failed");
    expect(shellLabel(ended("a", T))).toBe("Exited");
    expect(shellLabel(shell("a", { state: "quiet" }))).toBe("Quiet");
    expect(shellLabel(shell("a"))).toBe("Running");
  });

  it("puts the whole command, description, exit code and last line on hover", () => {
    expect(shellTitle(shell("a", { state: "failed", exitCode: 3, lastLine: "build broke" }))).toBe("npm run dev\nStart the dev server\nexit code 3\nlast: build broke");
    expect(shellTitle(shell("a", { description: "npm run dev" }))).toBe("npm run dev");
  });
});

describe("linger and fold", () => {
  it("lingers like a finished subagent", () => {
    expect(SHELL_LINGER_MS).toBe(KID_LINGER_MS);
    expect(SHELL_LINGER_MS).toBe(30_000);
  });

  it("keeps an exited chip for 30s, then counts it as ended", () => {
    const end = T - 1000;
    const shells = [shell("run"), ended("ex", end)];
    expect(planShells(shells, end + SHELL_LINGER_MS - 1)).toEqual({ shown: [shells[0], shells[1]], ended: 0 });
    const later = planShells(shells, end + SHELL_LINGER_MS);
    expect(later.shown.map((s) => s.id)).toEqual(["run"]);
    expect(later.ended).toBe(1);
  });

  it("orders live oldest first, then ended newest first; folded ones show only when opened", () => {
    const shells = [ended("old", T - 200_000), shell("b", { startedAt: T - 10 }), ended("new", T - 2000), shell("a", { startedAt: T - 99 }), ended("older", T - 300_000)];
    expect(planShells(shells, T).shown.map((s) => s.id)).toEqual(["a", "b", "new"]);
    expect(planShells(shells, T, true).shown.map((s) => s.id)).toEqual(["a", "b", "new", "old", "older"]);
  });

  it("schedules the next fold at the earliest lingering chip", () => {
    expect(nextShellFoldAt([shell("run"), ended("a", T - 5000), ended("b", T - 2000), ended("gone", T - 99_000)], T)).toBe(T - 5000 + SHELL_LINGER_MS);
    expect(nextShellFoldAt([shell("run")], T)).toBeNull();
    expect(nextShellFoldAt(undefined, T)).toBeNull();
  });

  it("folds a failed chip too, but the parent's dot outlives it", () => {
    const failed = ended("f", T - 1000, { state: "failed", exitCode: 1, command: "npm run build", lastLine: "error TS2307" });
    expect(planShells([failed], T + SHELL_LINGER_MS).shown).toEqual([]);
    expect(failNote([failed])).toBe("Shell failed: npm run build\nexit code 1\nlast: error TS2307");
  });
});

describe("failed dot", () => {
  const base: Session = { id: "s", pid: 1, name: "p", cwd: "C:/p", kind: "interactive", entrypoint: "cli", state: "working", stateSince: T, task: "", subagents: 0, alive: true };

  it("is off with no shells or none failed", () => {
    expect(failNote(undefined)).toBe("");
    expect(failNote([shell("a"), ended("b", T), shell("c", { state: "serving", port: 80 })])).toBe("");
    expect(toRowView(base).shellNote).toBe("");
  });

  it("lists every failed shell, newest first, and says when no code came back", () => {
    const note = failNote([
      ended("a", T - 9000, { state: "failed", exitCode: 2, command: "make", lastLine: undefined }),
      ended("b", T - 1000, { state: "failed", exitCode: undefined, command: "npm test" }),
    ]);
    expect(note).toBe("Shell failed: npm test\nno exit code\n\nShell failed: make\nexit code 2");
  });

  it("reaches the row view so the closed row and the strip can flag it", () => {
    const v = toRowView({ ...base, shells: [ended("b", T, { state: "failed", exitCode: 3 })] });
    expect(v.shellNote).toContain("exit code 3");
  });
});
