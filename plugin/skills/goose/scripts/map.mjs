#!/usr/bin/env node
// Duck Duck Goose: import map + standalone graph page. Usage: node map.mjs [repoRoot] [--graphify path/to/graph.json]
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SKIP = new Set(["node_modules", "target", ".git", "dist", "dist-ssr", ".peep", "graphify-out", "build", "coverage", "__pycache__", "vendor"]);
const EXTS = [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".rs"];
const PALETTE = ["yellow", "pink", "lavender", "blue", "green"];

const norm = (p) => p.replace(/\\/g, "/");

export function walk(root, dir = root, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP.has(e.name)) continue;
    const full = path.join(dir, e.name);
    if (e.isDirectory()) walk(root, full, out);
    else if (EXTS.includes(path.extname(e.name)) && !/\.d\.ts$/.test(e.name)) out.push(norm(path.relative(root, full)));
  }
  return out;
}

export function parseAreas(setupText) {
  const areas = [];
  for (const line of (setupText || "").split(/\r?\n/)) {
    const m = line.match(/^area\.([a-z0-9-]+):\s*(.+?)\s*$/);
    if (m) areas.push({ name: m[1], prefixes: m[2].split(",").map((s) => norm(s.trim())).filter(Boolean) });
    if (/^---\s*$/.test(line) && areas.length) break;
  }
  return areas;
}

export function areaOf(file, areas) {
  for (const a of areas) if (a.prefixes.some((p) => file.startsWith(p))) return a.name;
  return "other";
}

function resolveRel(from, spec, files) {
  const base = path.posix.normalize(path.posix.join(path.posix.dirname(from), spec));
  const stem = base.replace(/\.(m?js|cjs|jsx?)$/, "");
  for (const c of [base, ...EXTS.map((e) => stem + e), ...EXTS.map((e) => base + "/index" + e)]) if (files.has(c)) return c;
  return null;
}

function tsImports(src) {
  const out = [];
  const re = /(?:\bfrom\s*|\bimport\s*\(?\s*|\brequire\s*\(\s*|\bimport\s+)["']([^"']+)["']/g;
  let m;
  while ((m = re.exec(src))) if (m[1].startsWith(".")) out.push(m[1]);
  return out;
}

function crateRootOf(file, files) {
  let dir = path.posix.dirname(file);
  while (dir && dir !== ".") {
    if (files.has(dir + "/lib.rs") || files.has(dir + "/main.rs")) return dir;
    dir = path.posix.dirname(dir);
  }
  return null;
}

function rsModFile(dir, segs, files) {
  for (let n = segs.length; n > 0; n--) {
    const p = dir + "/" + segs.slice(0, n).join("/");
    if (files.has(p + ".rs")) return p + ".rs";
    if (files.has(p + "/mod.rs")) return p + "/mod.rs";
  }
  return null;
}

function rsRefs(file, src, files) {
  const out = [];
  const root = crateRootOf(file, files);
  const re = /\bcrate::((?:\{[^}]*\}|\w+)(?:::(?:\{[^}]*\}|\w+))*)/g;
  let m;
  while (root && (m = re.exec(src))) {
    const head = m[1].split("::");
    const first = head[0];
    const tails = first.startsWith("{") ? first.slice(1, -1).split(",").map((s) => s.trim().split("::")) : [head];
    for (const segs of tails) {
      const hit = rsModFile(root, segs.filter((s) => /^\w+$/.test(s)), files);
      if (hit) out.push(hit);
    }
  }
  const dir = path.posix.dirname(file);
  const stem = path.posix.basename(file, ".rs");
  const modDir = ["mod", "lib", "main"].includes(stem) ? dir : dir + "/" + stem;
  const mre = /^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/gm;
  while ((m = mre.exec(src))) {
    const hit = rsModFile(modDir, [m[1]], files);
    if (hit) out.push(hit);
  }
  return out;
}

export function buildImportMap(root, areas) {
  const list = walk(root).sort();
  const files = new Set(list);
  const edges = new Set();
  for (const f of list) {
    const src = fs.readFileSync(path.join(root, f), "utf8");
    const targets = f.endsWith(".rs") ? rsRefs(f, src, files) : tsImports(src).map((s) => resolveRel(f, s, files));
    for (const t of targets) if (t && t !== f) edges.add(f + "\t" + t);
  }
  return {
    nodes: list.map((id) => ({ id, area: areaOf(id, areas) })),
    edges: [...edges].map((e) => e.split("\t")),
  };
}

// graphify's graph.json is node_link: nodes carry source_file, links carry source/target node ids.
export function fromGraphify(json, root, areas) {
  const file = new Map();
  for (const n of json.nodes || []) if (n.source_file) file.set(n.id, norm(path.relative(root, path.resolve(root, n.source_file))));
  const nodes = new Map();
  const edges = new Set();
  const add = (f) => nodes.set(f, { id: f, area: areaOf(f, areas) });
  for (const l of json.links || json.edges || []) {
    const a = file.get(l.source), b = file.get(l.target);
    if (!a || !b || a === b) continue;
    add(a); add(b);
    edges.add(a + "\t" + b);
  }
  for (const f of file.values()) add(f);
  return { nodes: [...nodes.values()].sort((x, y) => x.id.localeCompare(y.id)), edges: [...edges].map((e) => e.split("\t")) };
}

export function callersOf(map, target) {
  return map.edges.filter(([, t]) => t === target).map(([f]) => f);
}

const EDIT_TOOLS = new Set(["Edit", "Write", "MultiEdit", "NotebookEdit"]);
const TAIL_BYTES = 2 << 20;
const EDIT_WINDOW_MS = 30 * 60 * 1000;

function readTail(file) {
  const fd = fs.openSync(file, "r");
  try {
    const size = fs.fstatSync(fd).size;
    const len = Math.min(size, TAIL_BYTES);
    const buf = Buffer.alloc(len);
    fs.readSync(fd, buf, 0, len, size - len);
    return buf.toString("utf8");
  } finally {
    fs.closeSync(fd);
  }
}

// Repo-relative files a transcript's Edit/Write calls touched at or after `since`.
export function editedFiles(transcript, root, since) {
  let text;
  try { text = readTail(transcript); } catch { return []; }
  const r = norm(path.resolve(root)).toLowerCase().replace(/\/$/, "");
  const out = new Set();
  for (const line of text.split("\n")) {
    if (!line.includes('"tool_use"')) continue;
    let d;
    try { d = JSON.parse(line); } catch { continue; }
    if (d.type !== "assistant" || !(Date.parse(d.timestamp) >= since)) continue;
    for (const c of d.message?.content || []) {
      const p = c.type === "tool_use" && EDIT_TOOLS.has(c.name) && (c.input?.file_path || c.input?.notebook_path);
      if (!p) continue;
      const abs = norm(path.resolve(root, p));
      if (abs.toLowerCase().startsWith(r + "/")) out.add(abs.slice(r.length + 1));
    }
  }
  return [...out];
}

export function liveSessions(root, areas, dir = path.join(os.homedir(), ".claude", "sessions"), projectsDir = path.join(os.homedir(), ".claude", "projects"), now = Date.now()) {
  const out = [];
  let names = [];
  try { names = fs.readdirSync(dir).filter((n) => n.endsWith(".json")); } catch { return out; }
  const r = norm(path.resolve(root)).toLowerCase().replace(/\/$/, "");
  for (const n of names) {
    let s;
    try { s = JSON.parse(fs.readFileSync(path.join(dir, n), "utf8")); } catch { continue; }
    if (!s.cwd) continue;
    const cwd = norm(s.cwd).replace(/\/$/, "");
    const low = cwd.toLowerCase();
    if (low !== r && !low.startsWith(r + "/")) continue;
    const rel = cwd.slice(r.length).replace(/^\//, "");
    const hash = [...cwd.toLowerCase()].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7);
    // Claude Code names the transcript folder after the cwd with every non-alphanumeric as "-".
    const transcript = s.sessionId ? path.join(projectsDir, s.cwd.replace(/[^A-Za-z0-9]/g, "-"), s.sessionId + ".jsonl") : "";
    const edited = transcript ? editedFiles(transcript, root, now - EDIT_WINDOW_MS) : [];
    out.push({
      name: s.name || String(s.pid),
      areas: edited.length ? [...new Set(edited.map((f) => areaOf(f, areas)))] : rel ? [areaOf(rel + "/", areas)] : [],
      state: s.status === "busy" ? "working" : s.status === "waiting" ? "waiting" : "idle",
      color: PALETTE[hash % PALETTE.length],
    });
  }
  return out;
}

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const json = (o) => JSON.stringify(o).replace(/</g, "\\u003c");

export function renderPage({ map, areas, sessions, peepJs, title }) {
  const order = [...areas.map((a) => a.name), "other"].filter((n) => map.nodes.some((x) => x.area === n));
  const data = { nodes: map.nodes, edges: map.edges, order, sessions };
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>${esc(title)} map</title>
<style>
:root{--desk:#0E0D12;--panel:#1B1921;--panel2:#262330;--line:rgba(255,255,255,.1);--text:#EDE9F2;--muted:#A49FB0;--accent:#F7D64A;--in:#7FE6FF;--out:#FFB23E}
@media (prefers-color-scheme:light){:root{--desk:#E6DDCD;--panel:#FFFCF6;--panel2:#F1E9DB;--line:rgba(42,34,51,.16);--text:#2A2233;--muted:#645C70;--accent:#8A5A00;--in:#0B7391;--out:#A85B00}}
*{box-sizing:border-box}
body{margin:0;background:var(--desk);color:var(--text);font:14px/1.4 "IBM Plex Sans",system-ui,sans-serif}
header{padding:12px 16px;border:2px solid var(--text);background:var(--panel);margin:16px;box-shadow:6px 6px 0 #000}
header h1{margin:0;font:600 18px "Fredoka",system-ui,sans-serif}
header p{margin:4px 0 0;color:var(--muted)}
main{display:grid;grid-template-columns:1fr 280px;gap:16px;padding:0 16px 24px;position:relative}
@media (max-width:800px){main{grid-template-columns:1fr}}
#board{position:relative;display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:16px;align-items:start}
.area{border:2px solid var(--text);background:var(--panel);box-shadow:6px 6px 0 #000}
.area h2{margin:0;padding:4px 8px;font:600 13px "IBM Plex Mono",monospace;background:repeating-linear-gradient(0deg,var(--panel2) 0 3px,var(--panel) 3px 6px);border-bottom:2px solid var(--text);display:flex;justify-content:space-between}
.peeps{display:flex;gap:4px;padding:4px 8px 0;min-height:0;flex-wrap:wrap}
.files{display:flex;flex-wrap:wrap;gap:3px;padding:8px}
.f{width:14px;height:14px;border:1px solid var(--muted);background:var(--panel2);padding:0;cursor:pointer}
.f:hover,.f:focus-visible{outline:2px solid var(--accent)}
.f.sel{background:var(--accent);border-color:var(--accent)}
.f.in{background:var(--in);border-color:var(--in)}
.f.out{background:var(--out);border-color:var(--out)}
.f.dim{opacity:.35}
aside{border:2px solid var(--text);background:var(--panel);padding:12px;align-self:start;position:sticky;top:16px;box-shadow:6px 6px 0 #000;max-height:90vh;overflow:auto}
aside h3{margin:0 0 6px;font:600 13px "IBM Plex Mono",monospace;word-break:break-all}
aside h4{margin:10px 0 2px;font-size:12px;color:var(--muted);text-transform:uppercase}
aside ul{margin:0;padding:0;list-style:none;font:12px "IBM Plex Mono",monospace}
aside li button{all:unset;cursor:pointer;word-break:break-all}
aside li button:hover{color:var(--accent)}
.key{color:var(--muted);font-size:12px;margin-top:8px}
</style></head><body>
<header><h1>${esc(title)}: who calls whom</h1><p id="count"></p></header>
<main><div id="board"></div>
<aside><h3 id="name">Pick a file</h3><div id="detail"><p class="key">Each square is a file. Click one to see what calls it (blue) and what it calls (orange).</p></div></aside></main>
<script>${peepJs}</script>
<script>
const D=${json(data)};
const byId=new Map(D.nodes.map(n=>[n.id,n]));
const callers=new Map(),callees=new Map();
for(const [a,b] of D.edges){(callees.get(a)||callees.set(a,[]).get(a)).push(b);(callers.get(b)||callers.set(b,[]).get(b)).push(a)}
document.getElementById('count').textContent=D.nodes.length+' files, '+D.edges.length+' imports, '+D.order.length+' areas';
const board=document.getElementById('board'),btn=new Map();
for(const area of D.order){
  const box=document.createElement('section');box.className='area';
  const h=document.createElement('h2');h.innerHTML='<span></span><span></span>';
  h.children[0].textContent=area;
  const mine=D.nodes.filter(n=>n.area===area);h.children[1].textContent=mine.length;
  box.append(h);
  const sess=D.sessions.filter(s=>s.areas.includes(area));
  if(sess.length){const p=document.createElement('div');p.className='peeps';
    for(const s of sess){const e=document.createElement('peep-bot');e.setAttribute('state',s.state);e.setAttribute('color',s.color);e.setAttribute('size','32');e.title=s.name;p.append(e)}
    box.append(p)}
  const fs=document.createElement('div');fs.className='files';
  for(const n of mine){const b=document.createElement('button');b.className='f';b.title=n.id;b.setAttribute('aria-label',n.id);b.onclick=()=>pick(n.id);btn.set(n.id,b);fs.append(b)}
  box.append(fs);board.append(box)}
const loose=D.sessions.filter(s=>!s.areas.some(a=>D.order.includes(a)));
if(loose.length){const p=document.createElement('div');p.className='area';p.innerHTML='<h2><span>repo root</span><span></span></h2>';const row=document.createElement('div');row.className='peeps';
  for(const s of loose){const e=document.createElement('peep-bot');e.setAttribute('state',s.state);e.setAttribute('color',s.color);e.setAttribute('size','32');e.title=s.name;row.append(e)}
  p.append(row);board.prepend(p)}
function list(title,ids){const wrap=document.createDocumentFragment();const h=document.createElement('h4');h.textContent=title+' ('+ids.length+')';wrap.append(h);
  const ul=document.createElement('ul');for(const id of ids){const li=document.createElement('li');const b=document.createElement('button');b.textContent=id;b.onclick=()=>pick(id);li.append(b);ul.append(li)}wrap.append(ul);return wrap}
function pick(id){
  const ins=callers.get(id)||[],outs=callees.get(id)||[];
  for(const [k,b] of btn){b.className='f'+(k===id?' sel':ins.includes(k)?' in':outs.includes(k)?' out':' dim')}
  document.getElementById('name').textContent=id;
  const d=document.getElementById('detail');d.textContent='';
  const a=document.createElement('p');a.className='key';a.textContent='area: '+byId.get(id).area;d.append(a,list('Called by',ins),list('Calls',outs))}
</script></body></html>
`;
}

export function run(root, opts = {}) {
  const setupPath = path.join(root, ".peep", "setup.md");
  const areas = fs.existsSync(setupPath) ? parseAreas(fs.readFileSync(setupPath, "utf8")) : [];
  const map = opts.graphify
    ? fromGraphify(JSON.parse(fs.readFileSync(opts.graphify, "utf8")), root, areas)
    : buildImportMap(root, areas);
  const here = path.dirname(fileURLToPath(import.meta.url));
  const peepJs = fs.readFileSync(path.join(here, "..", "assets", "peep.js"), "utf8").replace(/<\/script/gi, "<\\/script");
  const sessions = liveSessions(root, areas, opts.sessionsDir, opts.projectsDir);
  const outDir = path.join(root, ".peep", "graph");
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, "graph.json"), JSON.stringify(map, null, 1));
  fs.writeFileSync(path.join(outDir, "index.html"), renderPage({ map, areas, sessions, peepJs, title: path.basename(path.resolve(root)) }));
  return { outDir, files: map.nodes.length, edges: map.edges.length, sessions: sessions.length, source: opts.graphify ? "graphify" : "imports" };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const gi = args.indexOf("--graphify");
  const graphify = gi >= 0 ? args.splice(gi, 2)[1] : undefined;
  console.log(JSON.stringify(run(path.resolve(args[0] || "."), { graphify })));
}
