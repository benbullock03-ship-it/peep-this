import { expect, test } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { buildImportMap, callersOf, liveSessions, parseAreas, run } from "./map.mjs";

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
