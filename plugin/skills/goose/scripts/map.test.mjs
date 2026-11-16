import { expect, test } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { areaLinks, buildImportMap, callersOf, fixBrief, layoutAreas, liveSessions, openTweets, parseAreas, parsePatterns, peepLook, readLooks, renderPage, renderReport, run } from "./map.mjs";

test("area links fold file imports; layout puts needers left of what they need, cycles bounded", () => {
  const map = { nodes: [{ id: "a1", area: "ui" }, { id: "a2", area: "ui" }, { id: "b", area: "core" }, { id: "c", area: "io" }], edges: [["a1", "b"], ["a2", "b"], ["a1", "a2"], ["b", "c"]] };
  expect(areaLinks(map)).toEqual([["ui", "core"], ["core", "io"]]);
  expect(layoutAreas(["ui", "core", "io", "lone"], areaLinks(map))).toEqual({ ui: [0, 0], core: [1, 0], io: [2, 0], lone: [0, 1] });
  const cyc = layoutAreas(["x", "y"], [["x", "y"], ["y", "x"]]);
  expect(Math.max(cyc.x[0], cyc.y[0])).toBeLessThan(2);
});

test("only open tweets reach the graph page", () => {
  const board = "# Tweets\r\n\r\n## 1 · AV-1 · answered\r\nfrom: AV-1\r\nfiles: a\r\n\r\n## 2 · - · open\r\nfrom: ducks\r\nfiles: src/, docs/x.md\r\nask: who is in src?\r\n";
  expect(openTweets(board)).toEqual([{ from: "ducks", ask: "who is in src?", files: ["src/", "docs/x.md"] }]);
  const html = renderPage({ map: { nodes: [{ id: "src/a.ts", area: "frontend" }], edges: [] }, areas: [{ name: "frontend", prefixes: ["src/"] }], sessions: [], peepJs: "", title: "t", tweets: openTweets(board) });
  expect(html).toContain("Peep This · graph");
  expect(html).toContain("who is in src?");
});

test("import map links ts and rust files and finds callers", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "goose-"));
  const put = (f, s) => {
    fs.mkdirSync(path.dirname(path.join(root, f)), { recursive: true });
    fs.writeFileSync(path.join(root, f), s);
  };
  put("src/a.ts", 'import { b } from "./b";\n');
  put("src/b.ts", "export const b = 1;\n");
  put("rs/src/lib.rs", "mod state;\nmod mood;\n");
  put("rs/src/state.rs", "pub struct S;\n");
  put("rs/src/mood.rs", "use crate::state::S;\nuse crate::{state};\n");
  const areas = parseAreas("---\narea.web: src/\narea.core: rs/\n---\n");
  const map = buildImportMap(root, areas);
  expect(map.nodes.find((n) => n.id === "src/a.ts").area).toBe("web");
  expect(callersOf(map, "src/b.ts")).toEqual(["src/a.ts"]);
  expect(callersOf(map, "rs/src/state.rs").sort()).toEqual(["rs/src/lib.rs", "rs/src/mood.rs"]);
  fs.rmSync(root, { recursive: true, force: true });
});

test("a session's peep sits on the areas its recent edits fall in", () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "goose-"));
  const root = path.join(tmp, "repo w");
  const sessions = path.join(tmp, "sessions");
  const projects = path.join(tmp, "projects");
  fs.mkdirSync(root);
  fs.mkdirSync(sessions);
  const now = Date.parse("2026-11-16T10:00:00Z");
  const edit = (min, name, file_path) => JSON.stringify({
    type: "assistant", timestamp: new Date(now - min * 60000).toISOString(),
    message: { content: [{ type: "tool_use", name, input: { file_path } }] },
  });
  const session = (pid, sessionId, lines) => {
    fs.writeFileSync(path.join(sessions, pid + ".json"), JSON.stringify({ pid, sessionId, cwd: root, name: "s" + pid, status: "busy" }));
    const dir = path.join(projects, root.replace(/[^A-Za-z0-9]/g, "-"));
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, sessionId + ".jsonl"), lines.join("\n") + "\n");
  };
  session(1, "a", [
    edit(90, "Edit", path.join(root, "plugin", "old.mjs")),
    edit(5, "Edit", path.join(root, "src-tauri", "src", "state.rs")),
    edit(2, "Write", "src/new.ts"),
    edit(1, "Read", path.join(root, "docs", "spec.md")),
    edit(1, "Edit", path.join(tmp, "elsewhere.ts")),
  ]);
  session(2, "b", [edit(90, "Edit", path.join(root, "src", "a.ts"))]);
  const areas = parseAreas("---\narea.backend: src-tauri/\narea.frontend: src/\narea.plugin: plugin/\n---\n");
  const got = Object.fromEntries(liveSessions(root, areas, sessions, projects, now).map((s) => [s.name, s.areas]));
  expect(got).toEqual({ s1: ["backend", "frontend"], s2: [] });
  fs.rmSync(tmp, { recursive: true, force: true });
});

test("--graphify folds a graphify graph.json to file edges and writes the page", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "goose-"));
  fs.mkdirSync(path.join(root, ".peep"));
  fs.writeFileSync(path.join(root, ".peep", "setup.md"), "---\narea.backend: src-tauri/\narea.frontend: src/\n---\n");
  // Shape copied from a real graphify 0.9.66 graph.json (node_link_data, source_file repo-relative).
  const node = (id, source_file) => ({ id, label: id, file_type: "code", community: 0, source_file, source_location: "L1" });
  const link = (source, target) => ({ source, target, relation: "calls", confidence: "EXTRACTED", source_file: "x", weight: 1 });
  const graph = {
    directed: false, multigraph: false, graph: {}, hyperedges: [],
    nodes: [
      node("state_apply", "src-tauri/src/state.rs"),
      node("state_enter", "src-tauri/src/state.rs"),
      node("store_save", "src-tauri/src/store.rs"),
      node("main_render", "src/main.ts"),
      node("lonely", "src/lonely.ts"),
      { id: "concept", label: "concept" },
    ],
    links: [
      link("store_save", "state_apply"),
      link("state_apply", "state_enter"),
      link("main_render", "store_save"),
      link("concept", "state_apply"),
    ],
  };
  const gpath = path.join(root, "graph.json");
  fs.writeFileSync(gpath, JSON.stringify(graph));

  const res = run(root, { graphify: gpath, sessionsDir: path.join(root, "none") });
  const map = JSON.parse(fs.readFileSync(path.join(res.outDir, "graph.json"), "utf8"));

  expect(res).toMatchObject({ source: "graphify", files: 4, edges: 2, sessions: 0 });
  expect(callersOf(map, "src-tauri/src/state.rs")).toEqual(["src-tauri/src/store.rs"]);
  expect(callersOf(map, "src-tauri/src/store.rs")).toEqual(["src/main.ts"]);
  expect(map.nodes.find((n) => n.id === "src/lonely.ts").area).toBe("frontend");
  expect(fs.readFileSync(path.join(res.outDir, "index.html"), "utf8")).toContain("src-tauri/src/state.rs");
  fs.rmSync(root, { recursive: true, force: true });
});

test("a report puts candidate cards above the map, top pick first, with lesson, ELI5 and brief", () => {
  const md = fs.readFileSync(new URL("../../../references/patterns.md", import.meta.url), "utf8");
  const patterns = parsePatterns(md);
  expect(Object.keys(patterns)).toHaveLength(6);
  const report = {
    top: 1,
    commands: { build: "npm run build", test: "npm test" },
    cards: [
      { name: "A <b>", badge: "Speculative", pattern: "god-file", files: ["src/a.ts"], problem: "p1", fix: "f1" },
      { name: "Store seam", badge: "Strong", pattern: "leaky-seam", files: ["src/store.ts", "src/rows.ts"], problem: "rows read store internals", fix: "add a query function",
        before: { nodes: ["rows", "store"], edges: [[0, 1]] }, after: { nodes: ["rows", "query", "store"], edges: [[0, 1], [1, 2]] }, steps: ["add query()", "move reads"] },
    ],
  };
  const html = renderPage({ map: { nodes: [], edges: [] }, areas: [], sessions: [], peepJs: "", title: "t", report, patterns });
  expect(html.indexOf('id="report"')).toBeLessThan(html.indexOf('id="board"'));
  expect(html.indexOf("Store seam")).toBeLessThan(html.indexOf("A &lt;b&gt;"));
  expect(html).toContain(patterns["leaky-seam"].lesson);
  expect(html).toContain(patterns["god-file"].eli5);
  expect(html.match(/<details>/g)).toHaveLength(2);
  expect(html).toMatch(/state="stuck"[\s\S]*state="working"[\s\S]*state="success"/);
  expect(html).not.toMatch(/https?:\/\//);
  const brief = fixBrief(report.cards[1], report.commands);
  expect(brief).toContain("Files: src/store.ts, src/rows.ts");
  expect(brief).toContain("2. move reads");
  expect(brief).toContain("Check: npm run build && npm test");
  expect(renderReport({ cards: [] })).toBe("");
});

test("peepLook matches the widget's hashed identity and honours its look overrides", () => {
  expect(peepLook("F:\\Projects\\agentview widget")).toEqual({ color: "blue", accessory: "none" });
  expect(peepLook("/home/dev/app/")).toEqual({ color: "yellow", accessory: "shades" });
  const looks = { "62cc5ae9": { color: "pink", accessory: "supporter" } };
  expect(peepLook("C:/Users/Dev/Projects/AgentView", looks)).toEqual({ color: "pink", accessory: "supporter" });
  expect(peepLook("C:/Users/Dev/Projects/AgentView", { "62cc5ae9": { color: "red", accessory: "../x" } })).toEqual({ color: "yellow", accessory: "beanie" });
  const file = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "looks-")), "settings.json");
  fs.writeFileSync(file, JSON.stringify({ alertSound: true, peepLooks: looks }));
  expect(readLooks(file)).toEqual(looks);
  expect(readLooks(file + ".missing")).toEqual({});
});
