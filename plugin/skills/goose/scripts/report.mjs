#!/usr/bin/env node
// Goose report page. Usage: node report.mjs <map|health|topic> [repoRoot] [--graphify graph.json] [--data data.json]
// map computes its data from the repo; health and topic take the JSON the skill wrote (references/health.md, references/topic.md).
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { areaLinks, buildImportMap, fromGraphify, layoutAreas, liveSessions, parseAreas, parsePatterns } from "./map.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const TITLES = { map: "Code map", health: "Repo health", topic: "Custom topic" };

function git(root, ...args) {
  try { return execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim(); } catch { return ""; }
}

function meta(root) {
  return { repo: path.basename(path.resolve(root)), date: new Date().toISOString().slice(0, 10), commit: git(root, "rev-parse", "--short", "HEAD") };
}

export function mapData(root, { graphify, sessionsDir, projectsDir } = {}) {
  const setup = path.join(root, ".peep", "setup.md");
  const areas = fs.existsSync(setup) ? parseAreas(fs.readFileSync(setup, "utf8")) : [];
  const map = graphify ? fromGraphify(JSON.parse(fs.readFileSync(graphify, "utf8")), root, areas) : buildImportMap(root, areas);
  const order = [...areas.map((a) => a.name), "other"].filter((n) => map.nodes.some((x) => x.area === n));
  const edges = areaLinks(map);
  const at = layoutAreas(order, edges);
  const live = liveSessions(root, areas, sessionsDir, projectsDir);
  const editing = (id) => new Set(live.flatMap((s) => s.edited).filter((f) => map.nodes.some((n) => n.id === f && n.area === id))).size;
  const sessions = live.map((s) => ({ name: s.name, state: s.state, color: s.color, acc: s.accessory, area: s.areas[0] || "repo root" }));
  const busy = new Set(sessions.map((s) => s.area).filter((a) => order.includes(a))).size;
  const lead = `${order.length} area${order.length === 1 ? "" : "s"}, ${edges.length} link${edges.length === 1 ? "" : "s"}. ` +
    (sessions.length ? `${sessions.length} session${sessions.length === 1 ? " is" : "s are"} editing ${busy} area${busy === 1 ? "" : "s"}.` : "No session is editing this repo right now.");
  return {
    map,
    data: {
      ...meta(root), kind: "map", lead, peep: "idle",
      areas: order.map((id) => ({ id, col: at[id][0], row: at[id][1], files: map.nodes.filter((n) => n.area === id).length, editing: editing(id) })),
      edges, sessions, foot: [graphify ? "from graphify" : "from imports"],
    },
  };
}

// One pattern card per distinct pattern on the candidate cards, in card order.
export function patternCards(cards, patterns) {
  const seen = new Set();
  const out = [];
  for (const c of cards) {
    const p = patterns[c.pattern];
    if (!p || seen.has(c.pattern)) continue;
    seen.add(c.pattern);
    out.push({ lesson: p.lesson, why: c.problem, eli5: c.eli5 || p.eli5, source: p.source });
  }
  return out;
}

// The topic page's side map: the named areas, their links among themselves, laid out as the full map is.
export function sliceOf(full, ids) {
  const keep = full.areas.filter((a) => ids.includes(a.id));
  const names = keep.map((a) => a.id);
  const edges = full.edges.filter(([a, b]) => names.includes(a) && names.includes(b));
  const at = layoutAreas(names, edges);
  return { areas: keep.map((a) => ({ ...a, col: at[a.id][0], row: at[a.id][1] })), edges, note: `${keep.length} of ${full.areas.length} areas.` };
}

export function run(kind, root, { graphify, dataFile, ...rest } = {}) {
  if (!TITLES[kind]) throw new Error(`kind must be map, health or topic, got ${kind}`);
  const outDir = path.join(root, ".peep", "graph");
  fs.mkdirSync(outDir, { recursive: true });
  let data, extra = {};
  if (kind === "map") {
    const m = mapData(root, { graphify, ...rest });
    data = m.data;
    fs.writeFileSync(path.join(outDir, "graph.json"), JSON.stringify(m.map, null, 1));
    extra = { files: m.map.nodes.length, edges: m.map.edges.length, sessions: data.sessions.length, source: graphify ? "graphify" : "imports" };
  } else {
    if (!dataFile) throw new Error(`${kind} needs --data <file>`);
    data = { ...meta(root), foot: [], patterns: [], sessions: [], ...JSON.parse(fs.readFileSync(dataFile, "utf8")), kind };
    if (kind === "topic" && data.slice?.ids) data.slice = sliceOf(mapData(root, { graphify, ...rest }).data, data.slice.ids);
    if (kind === "health" && data.candidates) {
      fs.writeFileSync(path.join(outDir, "candidates.json"), JSON.stringify(data.candidates, null, 1));
      const patterns = parsePatterns(fs.readFileSync(path.join(here, "..", "..", "..", "references", "patterns.md"), "utf8"));
      data.patterns = patternCards(data.candidates.cards || [], patterns);
      delete data.candidates;
      extra = { cards: data.patterns.length };
    }
  }
  const template = fs.readFileSync(path.join(here, "..", "assets", "report.html"), "utf8");
  const json = JSON.stringify(data).replace(/</g, "\\u003c");
  const title = `Goose · ${TITLES[kind]} · ${data.repo}`;
  const page = template.replace("{{TITLE}}", () => title).replace("{{DATA}}", () => json);
  const file = path.join(outDir, kind + ".html");
  fs.writeFileSync(file, page);
  return { page: file, kind, ...extra };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const flag = (name) => { const i = args.indexOf(name); return i >= 0 ? args.splice(i, 2)[1] : undefined; };
  const graphify = flag("--graphify"), dataFile = flag("--data");
  const kind = args.shift();
  console.log(JSON.stringify(run(kind, path.resolve(args[0] || "."), { graphify, dataFile })));
}
