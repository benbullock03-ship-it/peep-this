#!/usr/bin/env node
// Serves .peep/sneak/<slug>/ : "/" shows every variant folder side by side, "/<variant>/" serves it.
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const TYPES = {
  ".html": "text/html; charset=utf-8", ".css": "text/css", ".js": "text/javascript",
  ".mjs": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml",
  ".png": "image/png", ".jpg": "image/jpeg", ".woff2": "font/woff2",
};

export function variants(root) {
  return fs.readdirSync(root, { withFileTypes: true })
    .filter((d) => d.isDirectory() && fs.existsSync(path.join(root, d.name, "index.html")))
    .map((d) => d.name).sort();
}

export function overview(root) {
  const cells = variants(root).map((v) =>
    `<section><h2>${v}</h2><iframe src="/${v}/" title="${v}"></iframe></section>`).join("");
  return `<!doctype html><meta charset="utf-8"><title>Sneak Peep</title>
<style>body{margin:0;font:14px system-ui;background:#222;color:#eee}
main{display:flex;gap:8px;padding:8px;height:calc(100vh - 16px)}
section{flex:1;display:flex;flex-direction:column;min-width:0}
h2{margin:0 0 6px;font-size:14px}iframe{flex:1;border:0;background:#fff}</style>
<main>${cells}</main>`;
}

export function createServer(root) {
  const base = path.resolve(root);
  return http.createServer((req, res) => {
    const url = decodeURIComponent(new URL(req.url, "http://x").pathname);
    if (url === "/") {
      res.writeHead(200, { "content-type": TYPES[".html"] });
      return res.end(overview(base));
    }
    let file = path.join(base, url);
    if (!file.startsWith(base + path.sep)) { res.writeHead(403); return res.end(); }
    if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
    if (!fs.existsSync(file)) { res.writeHead(404); return res.end("not found"); }
    res.writeHead(200, { "content-type": TYPES[path.extname(file)] || "application/octet-stream" });
    fs.createReadStream(file).pipe(res);
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const root = process.argv[2];
  if (!root || !fs.existsSync(root)) { console.error("usage: node serve.mjs <.peep/sneak/slug> [port]"); process.exit(1); }
  const server = createServer(root);
  server.listen(Number(process.argv[3]) || 0, "127.0.0.1", () =>
    console.log(`peep sneak: http://127.0.0.1:${server.address().port}/`));
}
