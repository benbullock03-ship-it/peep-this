import { describe, expect, it } from "vitest";
import { makeAnonymiser } from "./anonymise.mjs";

const anon = () =>
  makeAnonymiser({
    home: "C:\\Users\\Alice",
    user: "Alice",
    host: "DESKTOP-ABC123",
    cwds: ["F:\\Projects\\secret app"],
    names: ["secret-app-3f"],
  });

describe("makeAnonymiser", () => {
  it("replaces every spelling of the cwd, home, user and host", () => {
    const out = anon()({
      cwd: "F:\\Projects\\secret app",
      fwd: "F:/Projects/secret app/src/x.ts",
      bash: "cd /f/Projects/secret app && ls",
      transcript_path: "C:\\Users\\Alice\\.claude\\projects\\F--Projects-secret-app\\x.jsonl",
      pidDomain: "win32:desktop-abc123",
      who: "alice",
    });
    expect(out).toEqual({
      cwd: "C:\\work\\demo-1",
      fwd: "C:/work/demo-1/src/x.ts",
      bash: "cd /c/work/demo-1 && ls",
      transcript_path: "C:\\Users\\user\\.claude\\projects\\C--work-demo-1\\x.jsonl",
      pidDomain: "win32:test-host",
      who: "user",
    });
  });

  it("maps each uuid to one stable fake and renames sessions", () => {
    const a = anon();
    const id = "9F6FDFB2-9fcb-4370-9e50-18b03e879355";
    const first = a({ session_id: id, name: "secret-app-3f" });
    const second = a({ path: `x\\${id.toLowerCase()}.jsonl`, other: "11111111-2222-4333-8444-555555555555" });
    expect(first).toEqual({ session_id: "00000000-0000-4000-8000-000000000001", name: "session-1" });
    expect(second.path).toBe("x\\00000000-0000-4000-8000-000000000001.jsonl");
    expect(second.other).toBe("00000000-0000-4000-8000-000000000002");
  });

  it("redacts free text but keeps the shape and non-strings", () => {
    const out = anon()({
      prompt: "my private words",
      tool_input: { command: "ls", new_string: "code" },
      tool_response: { stdout: "out", interrupted: false, lines: [1, "x"] },
    });
    expect(out).toEqual({
      prompt: "<redacted>",
      tool_input: { command: "ls", new_string: "<redacted>" },
      tool_response: { stdout: "<redacted>", interrupted: false, lines: [1, "<redacted>"] },
    });
  });
});
