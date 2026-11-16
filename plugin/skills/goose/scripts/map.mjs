#!/usr/bin/env node
// Duck Duck Goose: import map + standalone graph page. Usage: node map.mjs [repoRoot] [--graphify path/to/graph.json]
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SKIP = new Set(["node_modules", "target", ".git", "dist", "dist-ssr", ".peep", "graphify-out", "build", "coverage", "__pycache__", "vendor"]);
const EXTS = [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".rs"];
const PALETTE = ["yellow", "pink", "lavender", "blue", "green"];
const ACCESSORIES = ["none", "hardhat", "headphones", "glasses", "bow", "party", "chef", "crybaby", "beanie", "crown", "shades", "flowers", "halo", "wizard"];

const norm = (p) => p.replace(/\\/g, "/");

// Same FNV-1a hash and normalisation as the widget's src/identity.ts, so a project looks the same here.
function fnv1a(s) {
  let h = 0x811c9dc5;
  for (const b of Buffer.from(s.trim().toLowerCase().replace(/\\/g, "/").replace(/\/+$/, ""), "utf8")) h = Math.imul(h ^ b, 0x01000193);
  return h >>> 0;
}

/** The widget's per-machine look overrides (AV-113), keyed by the cwd hash as 8 hex digits. */
export function readLooks(file = path.join(process.env.APPDATA || os.tmpdir(), "PeepThis", "settings.json")) {
  try { return JSON.parse(fs.readFileSync(file, "utf8")).peepLooks || {}; } catch { return {}; }
}

export function peepLook(cwd, looks = {}) {
  const h = fnv1a(cwd);
  const o = looks[h.toString(16).padStart(8, "0")] || {};
  return {
    color: PALETTE.includes(o.color) ? o.color : PALETTE[h % PALETTE.length],
    accessory: typeof o.accessory === "string" && /^[a-z]+$/.test(o.accessory) ? o.accessory : ACCESSORIES[(h >>> 16) % ACCESSORIES.length],
  };
}

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

export function liveSessions(root, areas, dir = path.join(os.homedir(), ".claude", "sessions"), projectsDir = path.join(os.homedir(), ".claude", "projects"), now = Date.now(), looks = readLooks()) {
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
    // Claude Code names the transcript folder after the cwd with every non-alphanumeric as "-".
    const transcript = s.sessionId ? path.join(projectsDir, s.cwd.replace(/[^A-Za-z0-9]/g, "-"), s.sessionId + ".jsonl") : "";
    const edited = transcript ? editedFiles(transcript, root, now - EDIT_WINDOW_MS) : [];
    out.push({
      name: s.name || String(s.pid),
      areas: edited.length ? [...new Set(edited.map((f) => areaOf(f, areas)))] : rel ? [areaOf(rel + "/", areas)] : [],
      edited,
      state: s.status === "busy" ? "working" : s.status === "waiting" ? "waiting" : "idle",
      ...peepLook(s.cwd, looks),
    });
  }
  return out;
}

// Area-level links: area a needs area b when a file in a imports a file in b.
export function areaLinks(map) {
  const area = new Map(map.nodes.map((n) => [n.id, n.area]));
  const out = new Set();
  for (const [f, t] of map.edges) {
    const a = area.get(f), b = area.get(t);
    if (a && b && a !== b) out.add(a + "\t" + b);
  }
  return [...out].map((l) => l.split("\t"));
}

// Column = longest chain of needs above the area (needers left, needed right); cycles stop growing at n - 1.
export function layoutAreas(order, links) {
  const col = Object.fromEntries(order.map((id) => [id, 0]));
  for (let pass = 0; pass < order.length; pass++) {
    let moved = false;
    for (const [a, b] of links) {
      if (col[a] == null || col[b] == null || col[b] > col[a] || col[a] + 1 >= order.length) continue;
      col[b] = col[a] + 1;
      moved = true;
    }
    if (!moved) break;
  }
  const rows = {};
  return Object.fromEntries(order.map((id) => {
    const c = col[id];
    rows[c] = (rows[c] ?? -1) + 1;
    return [id, [c, rows[c]]];
  }));
}

// The board's open tweets (formats.md §7): files only, for the selected-area panel.
export function openTweets(board) {
  return (board || "").replace(/\r\n/g, "\n").split("\n## ").slice(1).filter((b) => /·\s*open\s*$/.test(b.split("\n")[0])).map((b) => {
    const get = (k) => (b.match(new RegExp(`^${k}:\\s*(.*)$`, "m")) || [])[1] || "";
    return { from: get("from"), ask: get("ask"), files: get("files").split(",").map((f) => f.trim()).filter(Boolean) };
  });
}

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const json = (o) => JSON.stringify(o).replace(/</g, "\\u003c");

// Pattern rows of references/patterns.md: id -> { name, lesson, eli5, source }.
export function parsePatterns(md) {
  const out = {};
  for (const line of md.split("\n")) {
    const m = line.match(/^\|\s*`([a-z-]+)`\s*\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)\|/);
    if (m) out[m[1]] = { name: m[2].trim(), lesson: m[3].trim(), eli5: m[4].trim(), source: m[5].trim() };
  }
  return out;
}

export function renderPage({ map, areas, sessions, peepJs, title, tweets = [] }) {
  const order = [...areas.map((a) => a.name), "other"].filter((n) => map.nodes.some((x) => x.area === n));
  const links = areaLinks(map);
  const data = { nodes: map.nodes, order, links, at: layoutAreas(order, links), sessions, tweets, areas };
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>${esc(title)} map</title>
<style>
:root{--ps-desk:#0E0D12;--ps-panel:#1B1921;--ps-panel-2:#262330;--ps-edge:#4A4458;--ps-shelf:#4A3D5C;--ps-shelf-top:#6B5A82;--ps-text:#EDE9F2;--ps-muted:#A49FB0;--ps-accent:#F7D64A;--st-waiting:#FFB23E;--ps-line:rgba(255,255,255,.1);--ps-hard:rgba(0,0,0,.45);--in:#7FE6FF;--out:#FFB23E}
@media (prefers-color-scheme:light){:root{--ps-desk:#E6DDCD;--ps-panel:#FFFCF6;--ps-panel-2:#F1E9DB;--ps-edge:#2A2233;--ps-shelf:#C9B48E;--ps-shelf-top:#E2D2B0;--ps-text:#2A2233;--ps-muted:#645C70;--ps-accent:#8A5A00;--st-waiting:#A85B00;--ps-line:rgba(42,34,51,.16);--ps-hard:rgba(42,34,51,.35);--in:#0B7391;--out:#A85B00}}
:root{--desk:var(--ps-desk);--panel:var(--ps-panel);--panel2:var(--ps-panel-2);--line:var(--ps-line);--text:var(--ps-text);--muted:var(--ps-muted);--accent:var(--ps-accent);--gr-edge:var(--ps-edge);--gr-edge-hot:var(--ps-accent);--gr-file:var(--ps-edge);--gr-file-hot:var(--ps-accent);--shadow-hard:6px 6px 0 var(--ps-hard)}
*{box-sizing:border-box}
body{margin:0;padding:16px;background:var(--desk);color:var(--text);font:14px/1.4 "IBM Plex Sans",system-ui,sans-serif}
button{font:inherit;color:inherit;cursor:pointer}
.gnode:focus-visible,.links button:focus-visible{outline:2px solid var(--ps-accent);outline-offset:2px}
.gwin{width:max-content;max-width:100%;margin-top:16px;background:var(--ps-panel);border:2px solid var(--ps-edge);box-shadow:var(--shadow-hard);font:13px/1.4 "IBM Plex Mono",monospace}
.head{display:flex;align-items:center;gap:8px;height:34px;padding:0 6px 0 10px;border-bottom:2px solid var(--ps-edge);background:repeating-linear-gradient(0deg,transparent 0 3px,var(--ps-line) 3px 4px)}
.title{font-weight:600;color:var(--ps-accent);background:var(--ps-panel);padding:2px 6px;text-transform:uppercase;letter-spacing:.04em}
.sub{font-size:11px;color:var(--ps-muted);background:var(--ps-panel);padding:2px 6px;margin-right:auto}
.gbody{display:flex;overflow-x:auto}
.gmap{position:relative;flex:none;background:var(--ps-desk) repeating-linear-gradient(0deg,transparent 0 15px,var(--ps-line) 15px 16px)}
.gnode{position:absolute;width:120px;z-index:2;padding:0;border:0;background:none;text-align:left}
.gstand{display:flex;justify-content:center;align-items:flex-end;gap:4px;height:48px}
.plank{display:block;height:6px;background:repeating-linear-gradient(90deg,var(--ps-shelf) 0 8px,var(--ps-shelf-top) 8px 16px)}
.gcard{display:flex;flex-direction:column;gap:5px;height:40px;padding:5px 6px;background:var(--ps-panel);border:2px solid var(--ps-edge);border-top:0}
.gn{display:flex;justify-content:space-between;gap:6px;font:600 12px/14px "IBM Plex Mono",monospace;overflow:hidden;white-space:nowrap}
.gn span{font-weight:400;font-size:10px;color:var(--ps-muted)}
.gfiles{display:flex;gap:2px;height:4px;overflow:hidden}
.gfiles i{flex:none;width:4px;height:4px;background:var(--gr-file)}
.gfiles i.on{background:var(--gr-file-hot)}
.gnode.dim .gstand,.gnode.dim .plank,.gnode.dim .gfiles{opacity:.45}
.gnode.dim .gcard{color:var(--ps-muted);border-color:color-mix(in srgb,var(--ps-edge) 45%,var(--ps-desk))}
.gnode.hot .gcard{box-shadow:0 0 0 2px var(--gr-edge-hot)}
.gnode.sel .gcard{background:var(--ps-panel-2)}
.gnode.att .gcard{border-color:var(--st-waiting)}
.gedge{position:absolute;background:var(--gr-edge)}
.gedge.hot{background:var(--gr-edge-hot);z-index:1}
.gedge.dim{opacity:.4}
.gtip{position:absolute;z-index:3;width:250px;padding:6px 8px;background:var(--ps-panel);border:2px solid var(--ps-edge);box-shadow:var(--shadow-hard);font-size:11px;line-height:15px;pointer-events:none}
.gtip span,.gpanel .m{color:var(--ps-muted)}
.gpanel{flex:none;width:280px;border-left:2px solid var(--ps-edge);padding:12px;display:flex;flex-direction:column;gap:10px;font-size:12px}
.gpanel[hidden]{display:none}
.gpanel h3{margin:0;font:600 14px "IBM Plex Mono",monospace;word-break:break-all}
.gpanel .k{color:var(--ps-muted);font-size:10px;text-transform:uppercase;letter-spacing:.08em}
.links{display:flex;flex-wrap:wrap;gap:4px;margin-top:4px}
.links button{font-size:11px;padding:1px 6px;background:none;border:0;box-shadow:inset 0 0 0 1px var(--ps-muted)}
.links button:hover{box-shadow:inset 0 0 0 2px var(--ps-accent)}
.sess{display:flex;align-items:flex-end;gap:10px}
.gfoot{display:flex;flex-wrap:wrap;gap:6px 16px;padding:6px 10px;border-top:2px solid var(--ps-edge);font-size:11px;color:var(--ps-muted)}
</style></head><body>
<div class="gwin">
<div class="head"><span class="title">Peep This · graph</span><span class="sub" id="sub"></span></div>
<div class="gbody"><div class="gmap" id="board"></div><div class="gpanel" id="panel" hidden></div></div>
<div class="gfoot">Hover an area for its links. Click to pin it. Esc clears.</div>
</div>
<script>${peepJs}</script>
<script>
const D=${json(data)};
const map=document.getElementById('board'),panel=document.getElementById('panel'),PITCH=200,ROW=128,W=120;
const el=(t,c,txt)=>{const e=document.createElement(t);if(c)e.className=c;if(txt!=null)e.textContent=txt;return e};
const peep=(s,size)=>{const e=document.createElement('peep-bot');e.setAttribute('state',s.state);e.setAttribute('color',s.color);if(s.accessory)e.setAttribute('accessory',s.accessory);e.setAttribute('size',String(size));e.setAttribute('hover','off');e.title=s.name;return e};
const files=a=>D.nodes.filter(n=>n.area===a);
const prefixes=a=>(D.areas.find(x=>x.name===a)||{prefixes:[]}).prefixes;
const areaOf=f=>{for(const x of D.areas)if(x.prefixes.some(p=>f.startsWith(p)))return x.name;return 'other'};
const needs=a=>D.links.filter(l=>l[0]===a).map(l=>l[1]),usedBy=a=>D.links.filter(l=>l[1]===a).map(l=>l[0]);
const loose=D.sessions.filter(s=>!s.areas.some(a=>D.order.includes(a)));
const nodes=[...D.order];if(loose.length){nodes.unshift('repo root');D.at['repo root']=[0,Object.values(D.at).filter(p=>p[0]===0).length]}
const peepsIn=a=>a==='repo root'?loose:D.sessions.filter(s=>s.areas.includes(a));
const cols=Math.max(1,...nodes.map(a=>D.at[a][0]+1)),rows=Math.max(1,...nodes.map(a=>D.at[a][1]+1));
document.getElementById('sub').textContent=document.title.replace(/ map$/,'')+' · '+D.order.length+' areas · '+D.links.length+' links · '+D.sessions.length+' peeps';
const at={};let hover=null,pinned=null;
const bottom=14+(rows-1)*ROW+114;
map.style.width=20+(cols-1)*PITCH+W+40+'px';
for(const a of nodes){
  const [c,r]=D.at[a],x=20+c*PITCH,y=14+r*ROW,mine=a==='repo root'?[]:files(a),here=peepsIn(a);
  const hot=new Set(here.flatMap(s=>s.edited||[]).filter(f=>areaOf(f)===a)).size;
  const node=el('button','gnode'+(here.some(s=>s.state==='waiting')?' att':''));
  node.style.cssText='left:'+x+'px;top:'+y+'px';node.setAttribute('aria-label',a+', '+mine.length+' files');
  const stand=el('div','gstand');for(const s of here.slice(0,2))stand.append(peep(s,48));
  const card=el('div','gcard'),gn=el('div','gn',a);gn.append(el('span',null,String(mine.length)));
  const strip=el('div','gfiles'),blocks=Math.min(16,Math.ceil(mine.length/3)),on=Math.min(blocks,Math.ceil(hot/3));
  for(let i=0;i<blocks;i++)strip.append(el('i',i<on?'on':null));
  card.append(gn,strip);node.append(stand,el('span','plank'),card);
  node.onmouseenter=node.onfocus=()=>{hover=a;mark()};node.onmouseleave=node.onblur=()=>{hover=null;mark()};
  node.onclick=()=>{pinned=pinned===a?null:a;mark()};
  map.append(node);at[a]={id:a,x,y,r,n:mine.length,hot,peeps:here,node};
}
let far=0;
const seg=(x,y,w,h,a,b)=>{const s=el('span','gedge');s.style.cssText='left:'+x+'px;top:'+y+'px;width:'+w+'px;height:'+h+'px';s.dataset.a=a;s.dataset.b=b;map.append(s)};
const hseg=(xa,xb,y,a,b)=>seg(Math.min(xa,xb),y,Math.abs(xb-xa)+2,2,a,b),vseg=(x,ya,yb,a,b)=>seg(x,Math.min(ya,yb),2,Math.abs(yb-ya)+2,a,b);
for(const [a,b] of D.links){
  const A=at[a],B=at[b];if(!A||!B)continue;
  const x1=A.x+W,y1=A.y+73,y2=B.y+73,mx=x1+10+(A.r%5)*12;
  if(B.x-x1===PITCH-W){hseg(x1,mx,y1,a,b);vseg(mx,y1,y2,a,b);hseg(mx,B.x,y2,a,b)}
  else{
    // Not the next column: run in the gap under the target's row, so no line passes behind a card.
    const gy=B.y+116+(far%3)*4,bx=B.x-14-(far%3)*4;far++;
    hseg(x1,mx,y1,a,b);vseg(mx,y1,gy,a,b);hseg(mx,bx,gy,a,b);vseg(bx,gy,y2,a,b);hseg(bx,B.x,y2,a,b)}
  seg(B.x-6,y2-2,6,6,a,b);
}
map.style.height=bottom+(far?16:0)+'px';
function mark(){
  const focus=hover||pinned,near=focus?new Set([focus,...needs(focus),...usedBy(focus)]):null;
  for(const n of Object.values(at)){n.node.classList.toggle('dim',!!near&&!near.has(n.id));n.node.classList.toggle('hot',n.id===focus);n.node.classList.toggle('sel',n.id===pinned)}
  for(const s of map.querySelectorAll('.gedge')){const on=s.dataset.a===focus||s.dataset.b===focus;s.classList.toggle('hot',on);s.classList.toggle('dim',!!focus&&!on)}
  map.querySelector('.gtip')?.remove();
  if(hover&&hover!==pinned){const n=at[hover],tip=el('div','gtip');
    tip.append(el('b',null,n.id),el('br'),(prefixes(n.id).join(', ')||'')+' '+n.n+' files · '+n.hot+' being edited',el('br'),el('span',null,'needs '),(needs(hover).join(', ')||'nothing')+' · ',el('span',null,'used by '),usedBy(hover).join(', ')||'nothing');
    tip.style.cssText='left:'+(n.x+128)+'px;top:'+n.y+'px';map.append(tip)}
  panel.hidden=!pinned;if(!pinned)return;
  const n=at[pinned];panel.replaceChildren();
  const head=el('div');head.append(el('h3',null,n.id),el('div','m',(prefixes(n.id).join(', ')||'')+' · '+n.n+' files · '+n.hot+' being edited'));panel.append(head);
  const chips=(k,ids)=>{const w=el('div');w.append(el('div','k',k));const l=el('div','links');for(const id of ids){const b=el('button',null,id);b.onclick=()=>{pinned=id;mark()};l.append(b)}if(!ids.length)l.append(el('span','m','nothing'));w.append(l);panel.append(w)};
  chips('Needs',needs(pinned));chips('Used by',usedBy(pinned));
  panel.append(el('div','k','Peeps here'));
  for(const s of n.peeps){const r=el('div','sess'),t=el('span',null,s.name);t.append(el('br'),el('span','m',s.state));r.append(peep(s,48),t);panel.append(r)}
  if(!n.peeps.length)panel.append(el('span','m','No one is editing here.'));
  const tw=D.tweets.filter(t=>t.files.some(f=>areaOf(f)===n.id));
  if(tw.length){panel.append(el('div','k','Open tweets'));for(const t of tw)panel.append(el('div',null,t.from+': '+t.ask))}
}
addEventListener('keydown',e=>{if(e.key==='Escape'){pinned=null;mark()}});
mark();
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
  const board = path.join(root, ".peep", "tweets.md");
  const tweets = fs.existsSync(board) ? openTweets(fs.readFileSync(board, "utf8")) : [];
  fs.writeFileSync(path.join(outDir, "index.html"), renderPage({ map, areas, sessions, peepJs, title: path.basename(path.resolve(root)), tweets }));
  return { outDir, files: map.nodes.length, edges: map.edges.length, sessions: sessions.length, source: opts.graphify ? "graphify" : "imports" };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const flag = (name) => { const i = args.indexOf(name); return i >= 0 ? args.splice(i, 2)[1] : undefined; };
  const graphify = flag("--graphify");
  console.log(JSON.stringify(run(path.resolve(args[0] || "."), { graphify })));
}
