#!/usr/bin/env node
// PreToolUse hook: nudges a /peep:tweet on a clash or after two failures of one tool. Never blocks; fails silent.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { editedFiles } from "../skills/goose/scripts/map.mjs";

const EDIT_TOOLS = new Set(["Edit", "Write", "MultiEdit"]);
const CLASH_WINDOW_MS = 30 * 60 * 1000;
const TAIL_BYTES = 1 << 20;

const norm = (p) => p.replace(/\\/g, "/").replace(/\/$/, "");

export function repoRoot(cwd) {
  for (let d = path.resolve(cwd); ; d = path.dirname(d)) {
    if (fs.existsSync(path.join(d, ".git"))) return d;
    if (path.dirname(d) === d) return null;
  }
}

function inside(dir, root) {
  const a = norm(dir).toLowerCase(), r = norm(root).toLowerCase();
  return a === r || a.startsWith(r + "/");
}

function peers(root, ownId, sessionsDir) {
  let names = [];
  try { names = fs.readdirSync(sessionsDir).filter((n) => n.endsWith(".json")); } catch { return []; }
  const out = [];
  for (const n of names) {
    try {
      const s = JSON.parse(fs.readFileSync(path.join(sessionsDir, n), "utf8"));
      if (s.cwd && s.sessionId && s.sessionId !== ownId && inside(s.cwd, root)) out.push(s);
    } catch {}
  }
  return out;
}

function tail(file) {
  const fd = fs.openSync(file, "r");
  try {
    const size = fs.fstatSync(fd).size, len = Math.min(size, TAIL_BYTES), buf = Buffer.alloc(len);
    fs.readSync(fd, buf, 0, len, size - len);
    return buf.toString("utf8");
  } finally { fs.closeSync(fd); }
}

// The tool name and id of the last failure when the last two tool results both failed for one tool.
export function twoMisses(transcript) {
  let text;
  try { text = tail(transcript); } catch { return null; }
  const names = new Map(), results = [];
  for (const line of text.split("\n")) {
    if (!line.includes("tool_")) continue;
    let d;
    try { d = JSON.parse(line); } catch { continue; }
    for (const c of d.message?.content || []) {
      if (c.type === "tool_use") names.set(c.id, c.name);
      else if (c.type === "tool_result") results.push({ id: c.tool_use_id, err: c.is_error === true });
    }
  }
  const [a, b] = results.slice(-2);
  if (!a || !b || !a.err || !b.err) return null;
  const tool = names.get(b.id);
  return tool && names.get(a.id) === tool ? { tool, id: b.id } : null;
}

/** The additionalContext to give Claude, or null. `seen` holds this session's past nudge keys and is updated. */
export function nudge(input, { sessionsDir, projectsDir, now = Date.now(), seen = new Set() }) {
  const root = repoRoot(input.cwd || process.cwd());
  if (!root) return null;
  const others = peers(root, input.session_id, sessionsDir);
  const lines = [];

  const fp = input.tool_input?.file_path;
  if (EDIT_TOOLS.has(input.tool_name) && fp && others.length) {
    const abs = path.resolve(input.cwd || root, fp);
    const rel = norm(path.relative(root, abs));
    const key = "clash:" + rel.toLowerCase();
    if (!rel.startsWith("..") && !seen.has(key)) {
      const who = others.filter((s) => {
        const t = path.join(projectsDir, s.cwd.replace(/[^A-Za-z0-9]/g, "-"), s.sessionId + ".jsonl");
        return editedFiles(t, root, now - CLASH_WINDOW_MS).some((f) => f.toLowerCase() === rel.toLowerCase());
      });
      if (who.length) {
        seen.add(key);
        lines.push(`peep: another live session (${who.map((s) => s.name || s.pid).join(", ")}) edited ${rel} in the last 30 minutes. Before editing it, post a clash tweet with /peep:tweet naming the file and what you are about to change.`);
      }
    }
  }

  const miss = input.transcript_path && twoMisses(input.transcript_path);
  if (miss && !seen.has("stuck:" + miss.id)) {
    seen.add("stuck:" + miss.id);
    lines.push(others.length
      ? `peep: ${miss.tool} failed twice in a row. A live session shares this repo: post a stuck tweet with /peep:tweet before a third try.`
      : `peep: ${miss.tool} failed twice in a row. Rather than a third try, suggest the user /clear and start fresh, carrying over what was learned.`);
  }
  return lines.length ? lines.join("\n") : null;
}

function seenFile(sessionId) {
  return path.join(os.tmpdir(), "peep-tweet-nudge", String(sessionId).replace(/[^A-Za-z0-9-]/g, "_") + ".json");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const input = JSON.parse(fs.readFileSync(0, "utf8"));
    const file = seenFile(input.session_id);
    let seen = new Set();
    try { seen = new Set(JSON.parse(fs.readFileSync(file, "utf8"))); } catch {}
    const ctx = nudge(input, {
      sessionsDir: path.join(os.homedir(), ".claude", "sessions"),
      projectsDir: path.join(os.homedir(), ".claude", "projects"),
      seen,
    });
    if (ctx) {
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, JSON.stringify([...seen]));
      process.stdout.write(JSON.stringify({ hookSpecificOutput: { hookEventName: "PreToolUse", additionalContext: ctx } }));
    }
  } catch {}
  process.exit(0);
}
