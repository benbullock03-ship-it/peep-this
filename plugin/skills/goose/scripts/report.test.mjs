import { expect, test } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { patternCards, run, sliceOf } from "./report.mjs";

const data = (html) => JSON.parse(html.match(/<script type="application\/json" id="report">(.*?)<\/script>/s)[1]);

function repo() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "goose-report-"));
  const put = (f, s) => { fs.mkdirSync(path.dirname(path.join(root, f)), { recursive: true }); fs.writeFileSync(path.join(root, f), s); };
  put(".peep/setup.md", "area.ui: src/ui/\narea.core: src/core/\n---\n");
  put("src/ui/a.ts", 'import { b } from "../core/b";\n');
  put("src/core/b.ts", "export const b = 1;\n");
  return root;
}

test("map writes graph.json and a page with areas, links and no network", () => {
  const root = repo();
  const none = path.join(root, "none");
  const out = run("map", root, { sessionsDir: none, projectsDir: none });
  expect(out.kind).toBe("map");
  const html = fs.readFileSync(path.join(root, ".peep", "graph", "map.html"), "utf8");
  const d = data(html);
  expect(d.areas.map((a) => a.id)).toEqual(["ui", "core"]);
  expect(d.edges).toEqual([["ui", "core"]]);
  expect(d.sessions).toEqual([]);
  expect(html).not.toMatch(/(src|href)="https?:/);
  expect(fs.existsSync(path.join(root, ".peep", "graph", "graph.json"))).toBe(true);
});

test("health keeps candidates.json in the Ducks shape and turns its cards into pattern cards", () => {
  const root = repo();
  const candidates = { top: 0, commands: {}, cards: [{ name: "x", badge: "Strong", pattern: "god-file", files: ["a"], problem: "p", fix: "f" }] };
  const file = path.join(root, "h.json");
  fs.writeFileSync(file, JSON.stringify({ lead: "l", cards: [{ title: "Hot spots", clear: "none", rows: [] }], candidates }));
  run("health", root, { dataFile: file });
  const g = path.join(root, ".peep", "graph");
  expect(JSON.parse(fs.readFileSync(path.join(g, "candidates.json"), "utf8")).cards[0].pattern).toBe("god-file");
  const d = data(fs.readFileSync(path.join(g, "health.html"), "utf8"));
  expect(d.patterns).toHaveLength(1);
  expect(d.patterns[0].why).toBe("p");
  expect(patternCards([{ pattern: "nope" }], {})).toEqual([]);
});

test("topic slice keeps only the named areas and their links", () => {
  const full = { areas: [{ id: "a", col: 0, row: 0 }, { id: "b", col: 1, row: 0 }, { id: "c", col: 2, row: 0 }], edges: [["a", "b"], ["b", "c"]] };
  const s = sliceOf(full, ["a", "b"]);
  expect(s.edges).toEqual([["a", "b"]]);
  expect(s.note).toBe("2 of 3 areas.");
});
