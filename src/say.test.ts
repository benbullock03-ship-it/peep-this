import { describe, expect, it } from "vitest";
import type { PendingApproval, Session } from "./model";
import { sayFor } from "./say";

const T = 10_000_000;

const base: Session = {
  id: "s1",
  pid: 1,
  name: "agentview",
  cwd: "F:\\Projects\\agentview widget",
  kind: "interactive",
  entrypoint: "cli",
  state: "idle",
  stateSince: T,
  task: "Idle",
  subagents: 0,
  alive: true,
};

const pending = (tool: string, input: Record<string, unknown>, receivedAt: number): PendingApproval => ({
  id: `r${receivedAt}`,
  tool,
  input,
  cwd: "F:/p",
  receivedAt,
  releaseAt: receivedAt + 60_000,
  risk: { tier: "shell", reason: "shell command" },
});

describe("sayFor", () => {
  it("working names the running tool and target", () => {
    expect(sayFor({ ...base, state: "working", task: "Bash npm test", lastTool: { name: "Bash", target: "npm test" } }, T)).toBe("Running Bash npm test…");
    expect(sayFor({ ...base, state: "working", task: "Working" }, T)).toBe("Working…");
  });

  it("waiting says the notification text, else asks about the oldest held request", () => {
    expect(sayFor({ ...base, state: "waiting", task: "Claude needs your permission to use Bash" }, T)).toBe("Claude needs your permission to use Bash");
    const held = [pending("Read", { file_path: "F:/p/b.ts" }, 2), pending("Bash", { command: "npm run build" }, 1)];
    expect(sayFor({ ...base, state: "waiting", task: "Waiting for input", pending: held }, T)).toBe("Can I use Bash on npm run build?");
    expect(sayFor({ ...base, state: "waiting", task: "Waiting for input" }, T)).toBe("Waiting for you.");
  });

  it("error says the summary", () => {
    expect(sayFor({ ...base, state: "error", task: "API Error: 529 Overloaded" }, T)).toBe("API Error: 529 Overloaded");
    expect(sayFor({ ...base, state: "error", task: "Error" }, T)).toBe("Something went wrong.");
  });

  it("stuck counts minutes since the last event", () => {
    expect(sayFor({ ...base, state: "stuck", task: "No response for 6 min" }, T)).toBe("zzz… no reply for 6 min");
    expect(sayFor({ ...base, state: "stuck", stateSince: T - 125_000 }, T)).toBe("zzz… no reply for 8 min");
  });

  it("success, thinking and idle", () => {
    expect(sayFor({ ...base, state: "success", task: "Done" }, T)).toBe("All done!");
    expect(sayFor({ ...base, state: "thinking", task: "Thinking" }, T)).toBe("Hmm… thinking it over.");
    expect(sayFor({ ...base, state: "thinking", task: "Planning next step" }, T)).toBe("Planning next step");
    expect(sayFor(base, T)).toBe("Resting. Ready when you are.");
  });

  it("an ended session says goodbye whatever its state", () => {
    expect(sayFor({ ...base, state: "success", alive: false }, T)).toBe("Session closed. Bye!");
  });
});
