#!/usr/bin/env node
// Maintainer tool: rebuild assets/report.html, the offline report template, from the design files. Run in this repo: node plugin/skills/goose/scripts/bake.mjs
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..", "..", "..", "..");
const design = path.join(root, "design");
const fonts = path.join(root, "src", "assets", "fonts");
const read = (...p) => fs.readFileSync(path.join(...p), "utf8");
const b64 = (...p) => fs.readFileSync(path.join(...p)).toString("base64");

function tokens(file) {
  const css = read(file).replace(/\/\*[\s\S]*?\*\//g, "");
  const dark = css.match(/:root\s*\{([\s\S]*?)\n\}/);
  const light = css.match(/:root\[data-theme="light"\]\s*\{([\s\S]*?)\n\}/);
  return `:root {${dark[1]}\n}\n` + (light ? `@media (prefers-color-scheme: light) {\n:root {${light[1]}\n}\n}\n` : "");
}

const FACES = [["Fredoka", "600", "fredoka-latin-600-normal"], ["IBM Plex Sans", "400", "ibm-plex-sans-latin-400-normal"], ["IBM Plex Sans", "600", "ibm-plex-sans-latin-600-normal"],
  ["IBM Plex Mono", "400", "ibm-plex-mono-latin-400-normal"], ["IBM Plex Mono", "500 600", "ibm-plex-mono-latin-500-normal"]];
const sprite = (n) => "data:image/png;base64," + b64(design, "skill-pack-2", "sprites", n);
const parts = {
  FONTS: FACES.map(([fam, w, f]) => `@font-face { font-family: '${fam}'; font-weight: ${w}; src: url(data:font/woff2;base64,${b64(fonts, f + ".woff2")}) format('woff2'); }`).join("\n"),
  TOKENS: ["skill-pack", "skill-pack-2", "goose-report"].map((d) => tokens(path.join(design, d, "tokens.css"))).join(""),
  CSS: read(design, "goose-report", "goose.css"),
  PEEP: read(here, "..", "assets", "peep.js").replace(/<\/script/gi, "<\\/script"),
  JS: read(here, "report-client.js"),
  FLOPPY_DARK: sprite("floppies-dark.png"),
  FLOPPY_LIGHT: sprite("floppies-light.png"),
};
let page = read(design, "goose-report", "template.html");
for (const [k, v] of Object.entries(parts)) page = page.split("{{" + k + "}}").join(v);
const out = path.join(here, "..", "assets", "report.html");
fs.writeFileSync(out, page);
console.log(out, fs.statSync(out).size, "bytes");
