// npm run capture: records live Claude Code payloads, anonymised, into src-tauri/tests/fixtures/<version>/.
import { createServer, request } from "node:http";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir, hostname, userInfo } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { makeAnonymiser } from "./anonymise.mjs";

const WINDOW_MS = 60_000;
const PER_ROUTE = 3;
const fixtures = fileURLToPath(new URL("../src-tauri/tests/fixtures/", import.meta.url));
const claudeDir = process.env.CLAUDE_CONFIG_DIR || join(homedir(), ".claude");
const widgetDir = join(process.env.APPDATA || homedir(), "PeepThis");
const endpointPath = join(widgetDir, "endpoint.json");

const sessionsDir = join(claudeDir, "sessions");
const sessions = readdirSync(sessionsDir)
  .filter((f) => /^\d+\.json$/.test(f))
  .flatMap((f) => {
    try {
      return [{ file: f, json: JSON.parse(readFileSync(join(sessionsDir, f), "utf8")) }];
    } catch {
      return [];
    }
  });
if (!sessions.length) {
  console.error(`capture: no session files in ${sessionsDir}; start a Claude Code session first`);
  process.exit(1);
}

const semver = (v) => (v ?? "0").split(".").map(Number);
const newer = (a, b) => {
  const [x, y] = [semver(a), semver(b)];
  for (let i = 0; i < Math.max(x.length, y.length); i++) if ((x[i] ?? 0) !== (y[i] ?? 0)) return (x[i] ?? 0) > (y[i] ?? 0);
  return false;
};
const latest = sessions.map((s) => s.json.version).filter(Boolean).reduce((a, b) => (newer(b, a) ? b : a), "legacy");
const versionOf = new Map(sessions.map((s) => [s.json.sessionId, s.json.version || "legacy"]));

const anonymise = makeAnonymiser({
  home: homedir(),
  user: userInfo().username,
  host: hostname(),
  cwds: [...new Set(sessions.map((s) => s.json.cwd).filter(Boolean))],
  names: sessions.map((s) => s.json.name).filter(Boolean),
});

const written = new Set();
const out = (version, kind, name, value) => {
  const dir = join(fixtures, version, kind);
  if (!written.has(version)) {
    rmSync(join(fixtures, version), { recursive: true, force: true });
    written.add(version);
  }
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, name), JSON.stringify(anonymise(value), null, 2) + "\n");
};

for (const s of sessions) out(s.json.version || "legacy", "sessions", s.file, s.json);

// Proxy: the relay reads endpoint.json on every call, so pointing it here sees every POST while the widget still gets them.
const original = existsSync(endpointPath) ? readFileSync(endpointPath, "utf8") : null;
let target = null;
try {
  target = original && JSON.parse(original);
} catch {}
const token = target?.token || "0".repeat(64);
const counts = new Map();

const forward = (req, body) =>
  new Promise((resolve) => {
    if (!target?.port) return resolve({ status: 204, body: Buffer.alloc(0) });
    const fwd = request(
      { host: "127.0.0.1", port: target.port, method: req.method, path: req.url, headers: req.headers },
      (res) => {
        const chunks = [];
        res.on("data", (c) => chunks.push(c));
        res.on("end", () => resolve({ status: res.statusCode, body: Buffer.concat(chunks) }));
      },
    );
    fwd.on("error", () => resolve({ status: 204, body: Buffer.alloc(0) }));
    fwd.end(body);
  });

const record = (url, body) => {
  const route = url === "/statusline" ? "statusline" : url.match(/^\/hook\/([A-Za-z0-9]+)$/)?.[1];
  if (!route) return;
  let json;
  try {
    json = JSON.parse(body.toString("utf8"));
  } catch {
    return;
  }
  const n = (counts.get(route) ?? 0) + 1;
  counts.set(route, n);
  if (n > PER_ROUTE) return;
  out(versionOf.get(json.session_id) ?? latest, "hooks", `${route}-${n}.json`, json);
};

const server = createServer((req, res) => {
  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", async () => {
    const body = Buffer.concat(chunks);
    if (req.method === "POST") record(req.url ?? "", body);
    const reply = await forward(req, body);
    res.writeHead(reply.status);
    res.end(reply.body);
  });
});

let port = 0;
const restore = () => {
  try {
    const now = JSON.parse(readFileSync(endpointPath, "utf8"));
    // A widget that restarted meanwhile wrote its own endpoint; leave that one alone.
    if (now.port !== port) return;
    if (original === null) rmSync(endpointPath);
    else writeFileSync(endpointPath, original);
  } catch {}
};

const finish = () => {
  restore();
  server.close(() => {
    const hooks = [...counts].map(([r, n]) => `${r} ${Math.min(n, PER_ROUTE)}`).join(", ") || "none";
    console.log(`capture: wrote ${[...written].map((v) => `tests/fixtures/${v}/`).join(", ")}`);
    console.log(`capture: ${sessions.length} sessions; hooks: ${hooks}`);
    process.exit(0);
  });
};

server.listen(0, "127.0.0.1", () => {
  port = server.address().port;
  mkdirSync(widgetDir, { recursive: true });
  writeFileSync(endpointPath, JSON.stringify({ port, token }));
  if (!target?.port) console.log("capture: widget not running; hooks are recorded but not delivered");
  console.log(`capture: recording hooks for ${WINDOW_MS / 1000}s; use a Claude Code session now (Ctrl+C stops early)`);
  setTimeout(finish, WINDOW_MS);
});
process.on("SIGINT", finish);
