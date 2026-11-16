#!/usr/bin/env node
// Scrubadubdub pass 1. Usage: node scan.mjs [repoRoot] [--full] [--base <ref>] [--skip <prefix>]...
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { RULES, scan } from "../../../references/ai-tells/rules.mjs";

const SKIP_DIRS = new Set(["node_modules", "target", ".git", "dist", "dist-ssr", ".peep", "build", "coverage", "vendor", ".playwright-mcp"]);
const EXTS = [".css", ".scss", ".sass", ".less", ".html", ".htm", ".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".vue", ".svelte", ".astro"];
const WEIGHT = { P0: 3, P1: 2, P2: 1 };
const BY_ID = new Map(RULES.map((r) => [r.id, r]));
const HERE = path.dirname(fileURLToPath(import.meta.url));

const norm = (p) => p.replace(/\\/g, "/");

export function scannable(file, skips = []) {
  const parts = file.split("/");
  if (parts.slice(0, -1).some((p) => SKIP_DIRS.has(p))) return false;
  if (skips.some((s) => file.startsWith(s))) return false;
  if (/\.(?:test|spec|min)\.|\.d\.ts$/.test(file)) return false;
  return EXTS.includes(path.extname(file));
}

/** `.peep/scrub.md` (formats §6) as `{ earned: [{id, path}], debt: [{id, path, count}] }`. */
export function parseScrub(text) {
  const out = { earned: [], debt: [] };
  let section = null;
  for (const line of (text || "").split(/\r?\n/)) {
    const h = line.match(/^##\s+(\w+)/);
    if (h) { section = h[1].toLowerCase(); continue; }
    const m = line.match(/^-\s+([\w-]+)\s+·\s+(\S+)\s+·\s+(.+?)\s*$/);
    if (!m) continue;
    if (section === "earned") out.earned.push({ id: m[1], path: m[2] });
    if (section === "debt" && /^\d+$/.test(m[3])) out.debt.push({ id: m[1], path: m[2], count: Number(m[3]) });
  }
  return out;
}

const under = (file, p) => (p.endsWith("/") ? file.startsWith(p) : file === p);

/** Line sets touched since `base`, per file, from `git diff -U0`. */
export function parseDiff(diff) {
  const out = new Map();
  let file = null;
  for (const line of diff.split("\n")) {
    const f = line.match(/^\+\+\+ (?:b\/(.+)|\/dev\/null)$/);
    if (f) { file = f[1] ? norm(f[1]) : null; if (file && !out.has(file)) out.set(file, new Set()); continue; }
    const h = line.match(/^@@ -\S+ \+(\d+)(?:,(\d+))? @@/);
    if (h && file) for (let i = 0, n = h[2] === undefined ? 1 : Number(h[2]); i < n; i++) out.get(file).add(Number(h[1]) + i);
  }
  return out;
}

function walk(root, dir = root, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP_DIRS.has(e.name)) continue;
    const full = path.join(dir, e.name);
    if (e.isDirectory()) walk(root, full, out);
    else out.push(norm(path.relative(root, full)));
  }
  return out;
}

/** Earned and debt exceptions, then score: severity weight plus each other id in the same file. */
export function rank(hits, exceptions) {
  const kept = hits.filter((h) => !exceptions.earned.some((e) => e.id === h.id && under(h.file, e.path)));
  const debtNotes = [];
  const out = [];
  for (const d of exceptions.debt) {
    const mine = kept.filter((h) => h.id === d.id && under(h.file, d.path));
    if (mine.length !== d.count) debtNotes.push({ ...d, now: mine.length });
    // Debt may only shrink: over the count, every hit is reported.
    if (mine.length <= d.count) for (const h of mine) h.debt = true;
  }
  const idsByFile = new Map();
  for (const h of kept) {
    if (!idsByFile.has(h.file)) idsByFile.set(h.file, new Set());
    idsByFile.get(h.file).add(h.id);
  }
  // One finding per id per file, so eleven colour literals in one stylesheet are one number to approve.
  const groups = new Map();
  for (const h of kept) {
    if (h.debt) continue;
    const key = `${h.file}\0${h.id}`;
    if (groups.has(key)) { groups.get(key).lines.push(h.line); continue; }
    const rule = BY_ID.get(h.id);
    const together = [...idsByFile.get(h.file)].filter((id) => id !== h.id);
    const severity = together.length === 0 && rule.severity !== "P0" ? "P2" : rule.severity;
    groups.set(key, { file: h.file, line: h.line, lines: [h.line], id: h.id, group: rule.group, severity, together, score: WEIGHT[rule.severity] + together.length });
  }
  out.push(...groups.values());
  out.sort((a, b) => b.score - a.score || a.file.localeCompare(b.file) || a.line - b.line);
  return { findings: out.map((f, i) => ({ n: i + 1, ...f })), debt: debtNotes };
}

export function run(root, { full = false, base = "main", skips = [] } = {}) {
  const git = (...a) => execFileSync("git", a, { cwd: root, encoding: "utf8", maxBuffer: 64 << 20, stdio: ["ignore", "pipe", "ignore"] });
  // The pack's own catalogue and seeded page hit every rule by design.
  const own = ["../../../references/ai-tells", "../fixtures"].map((p) => norm(path.relative(root, path.resolve(HERE, p))) + "/");
  const allSkips = [...skips, ...own.filter((p) => !p.startsWith(".."))];
  let scope;
  if (full) scope = new Map(walk(root).map((f) => [f, null]));
  else {
    const has = (ref) => { try { git("rev-parse", "--verify", "-q", ref); return true; } catch { return false; } };
    if (!has(base)) base = ["master", "HEAD"].find(has) ?? base;
    scope = parseDiff(git("diff", "-U0", "--no-color", base));
    for (const f of git("ls-files", "--others", "--exclude-standard").split("\n").filter(Boolean)) scope.set(norm(f), null);
  }
  const hits = [];
  let scanned = 0;
  for (const [file, lines] of scope) {
    if (!scannable(file, allSkips) || !fs.existsSync(path.join(root, file))) continue;
    scanned++;
    for (const h of scan(fs.readFileSync(path.join(root, file), "utf8"))) if (!lines || lines.has(h.line)) hits.push({ file, ...h });
  }
  const scrubPath = path.join(root, ".peep", "scrub.md");
  const exceptions = parseScrub(fs.existsSync(scrubPath) ? fs.readFileSync(scrubPath, "utf8") : "");
  return { scope: full ? "full" : `since ${base}`, files: scope.size, scanned, ...rank(hits, exceptions) };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const opts = { full: false, base: "main", skips: [] };
  let root = process.cwd();
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--full") opts.full = true;
    else if (args[i] === "--base") opts.base = args[++i];
    else if (args[i] === "--skip") opts.skips.push(norm(args[++i]));
    else root = path.resolve(args[i]);
  }
  console.log(JSON.stringify(run(root, opts), null, 2));
}
