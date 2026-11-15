import { test, expect } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createServer } from "./serve.mjs";

test("serves the overview and each variant, blocks traversal", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "sneak-"));
  for (const v of ["a", "b", "c"]) {
    fs.mkdirSync(path.join(root, v));
    fs.writeFileSync(path.join(root, v, "index.html"), `<h1>${v}</h1>`);
  }
  const server = createServer(root);
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  const base = `http://127.0.0.1:${server.address().port}`;
  const home = await (await fetch(base + "/")).text();
  expect(home.match(/<iframe/g)).toHaveLength(3);
  expect(await (await fetch(base + "/b/")).text()).toBe("<h1>b</h1>");
  expect((await fetch(base + "/..%2f..%2fetc")).status).toBe(403);
  server.close();
  fs.rmSync(root, { recursive: true });
});
