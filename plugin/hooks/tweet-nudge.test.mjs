import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, expect, test } from "vitest";
import { nudge } from "./tweet-nudge.mjs";

const now = Date.parse("2026-11-16T10:00:00Z");
let tmp, root, sessionsDir, projectsDir;

beforeEach(() => {
  tmp = fs.mkdtempSync(path.join(os.tmpdir(), "nudge-"));
  root = path.join(tmp, "repo w");
  sessionsDir = path.join(tmp, "sessions");
  projectsDir = path.join(tmp, "projects");
  fs.mkdirSync(path.join(root, ".git"), { recursive: true });
  fs.mkdirSync(sessionsDir);
});
afterEach(() => fs.rmSync(tmp, { recursive: true, force: true }));

const line = (o) => JSON.stringify(o);
const editAt = (min, file) => line({
  type: "assistant", timestamp: new Date(now - min * 60000).toISOString(),
  message: { content: [{ type: "tool_use", id: "e" + min, name: "Edit", input: { file_path: path.join(root, file) } }] },
});
function session(pid, sessionId, lines) {
  fs.writeFileSync(path.join(sessionsDir, pid + ".json"), line({ pid, sessionId, cwd: root, name: "s" + pid }));
  const dir = path.join(projectsDir, root.replace(/[^A-Za-z0-9]/g, "-"));
  fs.mkdirSync(dir, { recursive: true });
  const t = path.join(dir, sessionId + ".jsonl");
  fs.writeFileSync(t, lines.join("\n") + "\n");
  return t;
}
const call = (id, name) => line({ type: "assistant", message: { content: [{ type: "tool_use", id, name, input: {} }] } });
const result = (id, err) => line({ type: "user", message: { content: [{ type: "tool_result", tool_use_id: id, is_error: err }] } });
const opts = (seen = new Set()) => ({ sessionsDir, projectsDir, now, seen });
const edit = (file, extra = {}) => ({ session_id: "me", cwd: root, tool_name: "Edit", tool_input: { file_path: file }, ...extra });

test("clash: a peer edited the same file in the last 30 minutes, nudged once", () => {
  session(1, "me", []);
  session(2, "peer", [editAt(5, "src/state.ts")]);
  const seen = new Set();
  expect(nudge(edit("src/state.ts"), opts(seen))).toMatch(/s2\) edited src\/state.ts.*clash tweet/);
  expect(nudge(edit(path.join(root, "src", "state.ts")), opts(seen))).toBeNull();
});

test("no clash on another file, and a stale edit does not count", () => {
  session(1, "me", []);
  session(2, "peer", [editAt(5, "src/other.ts"), editAt(45, "src/state.ts")]);
  expect(nudge(edit("src/state.ts"), opts())).toBeNull();
});

test("own edits never count as a clash", () => {
  session(1, "me", [editAt(1, "src/state.ts")]);
  expect(nudge(edit("src/state.ts"), opts())).toBeNull();
});

test("stuck with a live peer suggests a stuck tweet", () => {
  const t = session(1, "me", [call("a", "Bash"), result("a", true), call("b", "Bash"), result("b", true)]);
  session(2, "peer", []);
  const input = { session_id: "me", cwd: root, tool_name: "Bash", tool_input: {}, transcript_path: t };
  const seen = new Set();
  expect(nudge(input, opts(seen))).toMatch(/Bash failed twice.*stuck tweet/);
  expect(nudge(input, opts(seen))).toBeNull();
});

test("stuck alone suggests /clear; a success or a different tool between is not stuck", () => {
  const t = session(1, "me", [call("a", "Bash"), result("a", true), call("b", "Bash"), result("b", true)]);
  const input = { session_id: "me", cwd: root, tool_name: "Bash", tool_input: {}, transcript_path: t };
  expect(nudge(input, opts())).toMatch(/\/clear/);
  const t2 = session(1, "me", [call("a", "Bash"), result("a", true), call("b", "Edit"), result("b", true)]);
  expect(nudge({ ...input, transcript_path: t2 }, opts())).toBeNull();
  const t3 = session(1, "me", [call("a", "Bash"), result("a", true), call("b", "Bash"), result("b", false)]);
  expect(nudge({ ...input, transcript_path: t3 }, opts())).toBeNull();
});

test("outside a git repo or with unreadable files it stays silent", () => {
  expect(nudge({ session_id: "me", cwd: tmp, tool_name: "Edit", tool_input: { file_path: "x" } }, opts())).toBeNull();
  expect(nudge(edit("src/a.ts", { transcript_path: path.join(tmp, "missing.jsonl") }), { ...opts(), sessionsDir: path.join(tmp, "nope") })).toBeNull();
});
