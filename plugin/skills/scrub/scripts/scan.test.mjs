import { expect, test } from "vitest";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { parseDiff, parseScrub, rank, run, scannable } from "./scan.mjs";

const SEEDED = fs.readFileSync(new URL("../fixtures/seeded.html", import.meta.url), "utf8");
const FIVE = ["default-face", "purple-gradient", "gradient-text", "tracked-caps", "glass"];

function repo(files) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "scrub-"));
  const git = (...a) => execFileSync("git", a, { cwd: root, stdio: "pipe" });
  git("init", "-q", "-b", "main");
  git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "init");
  for (const [f, s] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, f)), { recursive: true });
    fs.writeFileSync(path.join(root, f), s);
  }
  return root;
}

test("seeded page: all five hallmarks found, ranked as a cluster", () => {
  const root = repo({ "web/index.html": SEEDED, "vendor/x.css": "a { backdrop-filter: blur(2px); }" });
  const out = run(root);
  expect(out.findings.map((f) => f.id).sort()).toEqual([...FIVE].sort());
  expect(out.findings.every((f) => f.together.length === 4)).toBe(true);
  expect(out.findings[0].severity).toBe("P0");
  fs.rmSync(root, { recursive: true, force: true });
});

test("a fix removes only that finding on re-scan", () => {
  const root = repo({ "web/index.html": SEEDED });
  const page = path.join(root, "web/index.html");
  fs.writeFileSync(page, SEEDED.replace('"Inter"', '"IBM Plex Sans"').replace("backdrop-filter: blur(12px)", "background: var(--panel)"));
  expect(run(root).findings.map((f) => f.id).sort()).toEqual(["gradient-text", "purple-gradient", "tracked-caps"]);
  fs.rmSync(root, { recursive: true, force: true });
});

test("diff scope keeps only changed lines; --full sees committed ones", () => {
  const root = repo({ "a.css": "a { backdrop-filter: blur(2px); }\n" });
  execFileSync("git", ["add", "."], { cwd: root });
  execFileSync("git", ["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "a"], { cwd: root });
  fs.appendFileSync(path.join(root, "a.css"), "b { -webkit-background-clip: text; }\n");
  expect(run(root).findings.map((f) => f.id)).toEqual(["gradient-text"]);
  expect(run(root, { full: true }).findings.map((f) => f.id).sort()).toEqual(["glass", "gradient-text"]);
  fs.rmSync(root, { recursive: true, force: true });
});

test("single non-P0 hit is capped at P2", () => {
  const { findings } = rank([{ file: "a.css", line: 1, id: "glass" }], { earned: [], debt: [] });
  expect(findings[0]).toMatchObject({ severity: "P2", together: [] });
});

test("earned suppresses; debt suppresses at or under its count, reports over it", () => {
  const ex = parseScrub("## Earned\n- glass · src/ · Frosted panel is the brand.\n\n## Debt\n- tracked-caps · a.css · 1\n");
  expect(ex).toEqual({ earned: [{ id: "glass", path: "src/" }], debt: [{ id: "tracked-caps", path: "a.css", count: 1 }] });
  const one = rank([{ file: "src/x.css", line: 1, id: "glass" }, { file: "a.css", line: 2, id: "tracked-caps" }], ex);
  expect(one.findings).toEqual([]);
  const two = rank([{ file: "a.css", line: 2, id: "tracked-caps" }, { file: "a.css", line: 9, id: "tracked-caps" }], ex);
  expect(two.findings).toHaveLength(2);
  expect(two.debt).toEqual([{ id: "tracked-caps", path: "a.css", count: 1, now: 2 }]);
  expect(rank([], ex).debt).toEqual([{ id: "tracked-caps", path: "a.css", count: 1, now: 0 }]);
});

test("parseDiff and scannable", () => {
  const d = parseDiff("+++ b/s.css\n@@ -1,0 +3,2 @@\n+x\n+y\n+++ /dev/null\n@@ -1 +0,0 @@\n");
  expect([...d.get("s.css")]).toEqual([3, 4]);
  expect(scannable("src/a.css")).toBe(true);
  expect(scannable("src/a.test.ts")).toBe(false);
  expect(scannable("node_modules/x/a.css")).toBe(false);
  expect(scannable("design/a.css", ["design/"])).toBe(false);
});
