import { describe, expect, it } from "vitest";
import type { Agent, PendingApproval, Session } from "./model";
import { describeAsDoing, kidSayFor, sayFor, shortCommand } from "./say";

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
    expect(sayFor({ ...base, state: "working", task: "Edit src/a.ts", lastTool: { name: "Edit", target: "src/a.ts" } }, T)).toBe("Running Edit src/a.ts…");
    expect(sayFor({ ...base, state: "working", task: "Working" }, T)).toBe("Working…");
  });

  it("a shell call says its description, else program and first argument", () => {
    const bash = (target: string, description?: string): Session => ({ ...base, state: "working", task: `Bash ${target}`, lastTool: { name: "Bash", target, description } });
    expect(sayFor(bash("npm test -- --run", "Run the test suite"), T)).toBe("Running the test suite…");
    expect(sayFor(bash(".venv/Scripts/python - <<'EOF'"), T)).toBe("Running python…");
    expect(sayFor(bash("npm test"), T)).toBe("Running npm test…");
    expect(sayFor({ ...base, state: "working", lastTool: { name: "PowerShell", target: "Get-ChildItem -Recurse" } }, T)).toBe("Running Get-ChildItem…");
  });

  it("waiting asks about the oldest held request, else says the notification text", () => {
    expect(sayFor({ ...base, state: "waiting", task: "Claude needs your permission to use Bash" }, T)).toBe("Claude needs your permission to use Bash");
    const held = [pending("Read", { file_path: "F:/p/b.ts" }, 2), pending("Bash", { command: "npm run build" }, 1)];
    expect(sayFor({ ...base, state: "waiting", task: "Waiting for input", pending: held }, T)).toBe("Can I run npm run build?");
    expect(sayFor({ ...base, state: "waiting", task: "Needs permission: Bash npm run build", pending: held }, T)).toBe("Can I run npm run build?");
    const described = [pending("Bash", { command: "mkdir zz-test", description: "Create zz-test directory" }, 1)];
    expect(sayFor({ ...base, state: "waiting", task: "Waiting for input", pending: described }, T)).toBe("Can I create zz-test directory?");
    expect(sayFor({ ...base, state: "waiting", task: "Waiting for input" }, T)).toBe("Waiting for you.");
    const asked = { ...pending("AskUserQuestion", { questions: [{ question: "Which colour?\nPick one." }] }, 1), risk: { tier: "question" as const, reason: "" } };
    expect(sayFor({ ...base, state: "waiting", task: "Claude needs your permission", pending: [asked] }, T)).toBe("Which colour?");
  });

  it("a held shell request asks about the flagged part of the command", () => {
    const cmd = "git fetch origin && git reset --hard origin/main";
    const at = cmd.indexOf("git reset --hard");
    const p = { ...pending("Bash", { command: cmd }, 1), risk: { tier: "destructive" as const, field: "command", span: [at, at + 16] as [number, number], reason: "reset" } };
    expect(sayFor({ ...base, state: "waiting", task: "Waiting for input", pending: [p] }, T)).toBe("Can I run git reset?");
  });

  it("a permission notification for a shell call never shows the full command", () => {
    const s: Session = { ...base, state: "waiting", task: "Needs permission: Bash cat x | python - <<'EOF'", lastTool: { name: "Bash", target: "cat x | python - <<'EOF'" } };
    expect(sayFor(s, T)).toBe("Can I run cat x?");
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

describe("shortCommand", () => {
  it.each([
    [".venv/Scripts/python - <<'EOF'\nprint(1)\nEOF", "python"],
    ["npm test", "npm test"],
    ["npm run build -- --watch", "npm run build"],
    ["python -m pytest -q", "python -m pytest"],
    ["git log --oneline | head -5", "git log"],
    ["cd src-tauri && cargo test state", "cargo test"],
    ["NODE_ENV=test FOO=1 npx vitest run", "npx vitest"],
    ['"C:/Program Files/nodejs/node.exe" scripts/build.mjs', "node build.mjs"],
    ["C:\\tools\\rg.exe -n TODO src", "rg"],
    ["sudo apt-get install -y jq", "apt-get install"],
    ["echo \"hello world\" > out.txt", "echo"],
    ["cat a.txt; rm a.txt", "cat a.txt"],
    ["ls -la", "ls"],
    ["", ""],
  ])("%s -> %s", (cmd, want) => {
    expect(shortCommand(cmd)).toBe(want);
  });
});

describe("describeAsDoing", () => {
  it.each([
    ["Run the test suite", "Running the test suite"],
    ["Create zz-test directory", "Creating zz-test directory"],
    ["Wait 5 seconds", "Waiting 5 seconds"],
    ["Show working tree status.", "Showing working tree status"],
    ["Type-check the frontend", "Type-checking the frontend"],
    ["See what changed", "Seeing what changed"],
    ["Checking ports", "Checking ports"],
    ["Stop the dev server", "Stopping the dev server"],
  ])("%s -> %s", (desc, want) => {
    expect(describeAsDoing(desc)).toBe(want);
  });
});

describe("kidSayFor shell calls", () => {
  const kid: Agent = { id: "a1", agentType: "Explore", description: "scan", state: "working", stateSince: T, task: "Bash x", startedAt: T, toolCount: 1 };
  it("uses the description, else the short command", () => {
    expect(kidSayFor({ ...kid, lastTool: { name: "Bash", target: "grep -rn foo . | wc -l", description: "Count foo uses" } }, undefined, T)).toBe("Counting foo uses…");
    expect(kidSayFor({ ...kid, lastTool: { name: "Bash", target: "grep -rn foo . | wc -l" } }, undefined, T)).toBe("Running grep…");
  });
  it("asks about a held shell request without the command", () => {
    const held = [pending("Bash", { command: "rm -rf build && npm run build" }, 1)];
    expect(kidSayFor({ ...kid, state: "waiting", task: "Waiting for input" }, held, T)).toBe("Can I run rm?");
  });
});
