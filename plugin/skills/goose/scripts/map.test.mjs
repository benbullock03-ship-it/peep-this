import { expect, test } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { buildImportMap, callersOf, parseAreas } from "./map.mjs";

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
