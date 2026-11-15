import { describe, expect, it } from "vitest";
import fs from "node:fs";
import { RULES, scan } from "./rules.mjs";

// One snippet that must hit the rule and one that must hit nothing at all.
const FIXTURES = {
  "purple-gradient": {
    bad: ".hero { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }",
    good: ".hero { background: linear-gradient(180deg, var(--ground), var(--ground-2)); }",
  },
  "violet-accent": { bad: '<button class="bg-indigo-600 px-4">Save</button>', good: '<button class="bg-amber-400 px-4">Save</button>' },
  "gradient-text": {
    bad: "h1 { -webkit-background-clip: text; color: transparent; }",
    good: "h1 { background-clip: padding-box; color: var(--ink); }",
  },
  "colour-glow": {
    bad: ".card { box-shadow: 0 0 24px rgba(139, 92, 246, 0.5); }",
    good: ".btn:focus-visible { box-shadow: 0 0 0 2px var(--ring); } .card { box-shadow: 0 0 24px var(--shade); }",
  },
  "cream-terracotta": { bad: "--ground: #F4F1EA;\n--accent: #D97757;", good: "--ground: #141218;\n--accent: #F7D64A;" },
  "repeating-stripes": {
    bad: ".bar { background: repeating-linear-gradient(0deg, transparent 0 3px, var(--line) 3px 4px); }",
    good: ".bar { background: var(--panel); }",
  },
  "default-face": {
    bad: 'body { font-family: "Inter", sans-serif; }',
    good: 'body { font-family: "IBM Plex Sans", system-ui, sans-serif; } .x { font-family: Interstate, serif; }',
  },
  "tasteful-face": { bad: 'h1 { font-family: "Instrument Serif", serif; }', good: 'h1 { font-family: "Fredoka", sans-serif; }' },
  "tracked-caps": {
    bad: ".label { text-transform: uppercase; letter-spacing: .08em; }",
    good: ".label { text-transform: none; letter-spacing: 0.02em; }",
  },
  "neg-tracking": { bad: "h1 { letter-spacing: -0.02em; }", good: "h1 { letter-spacing: 0; }" },
  "accent-word": { bad: "<h1>Ship code that <em>just works</em></h1>", good: "<h1>Ship code that works</h1><p>It <em>does</em>.</p>" },
  eyebrow: { bad: '<p class="section-eyebrow">Features</p>', good: '<p class="crumb">Features</p>' },
  "stat-strip": {
    bad: "<li>10K+ users</li><li>99.9% uptime</li><li>4.9★ rating</li>",
    good: "<li>3 sessions running</li><li>48% of the 5-hour limit</li>",
  },
  "numbered-markers": {
    bad: "<span>01</span><h3>Fast</h3><span>02</span><h3>Safe</h3><span>03</span><h3>Easy</h3>",
    good: "<span>01</span><h3>Install</h3><td>2026-01-05</td>",
  },
  "default-shell": {
    bad: '<section class="container mx-auto px-4"></section><section class="container mx-auto px-4"></section><div class="max-w-7xl mx-auto"></div>',
    good: '<section class="container mx-auto px-4"></section><section class="w-full"></section>',
  },
  "most-popular": { bad: '<span class="tag">Most popular</span>', good: '<span class="tag">Team plan</span>' },
  "shadcn-radius": { bad: ":root { --radius: 0.5rem; }", good: ":root { --radius: 0; }" },
  "card-kit": { bad: '<div class="rounded-2xl border shadow-lg p-6">', good: '<div class="rounded-none border p-6">' },
  glass: { bad: ".nav { backdrop-filter: blur(12px); }", good: ".nav { background: var(--panel); }" },
  "side-stripe": { bad: ".note { border-left: 4px solid var(--accent); }", good: ".note { border-left: 1px solid var(--line); }" },
  "headline-pill": {
    bad: '<span class="badge">New: v2</span>\n<h1>Peep This</h1>',
    good: '<h1>Peep This</h1>\n<span class="badge">v2</span>',
  },
  "pill-radius": { bad: ".chip { border-radius: 999px; }", good: ".chip { border-radius: 0; }" },
  "effect-library": { bad: "<Spotlight className=\"top-0\" />", good: "<SpotlightSearch open={open} />" },
  "emoji-icons": { bad: "<li>🚀 Fast</li><li>🔒 Secure</li>", good: "<li>Fast</li><li>Secure</li><small>© 2026</small>" },
  "worn-icons": { bad: 'import { Sparkles, Check } from "lucide-react";', good: 'import { Folder } from "lucide-react";' },
  "fake-window": { bad: ".dot-red { background: #ff5f57; }", good: ".dot { background: var(--st); }" },
  "default-dark-card": { bad: '<div class="bg-zinc-950 border border-white/10">', good: '<div class="bg-zinc-900 border border-zinc-700">' },
  "hard-shadow": {
    bad: ".card { box-shadow: 6px 6px 0 rgba(0, 0, 0, .45); }",
    good: ".card { box-shadow: 0 1px 3px var(--shade); } .b:focus-visible { box-shadow: 0 0 0 2px var(--ring); } .p { box-shadow: 0 3px 0 var(--shade); }",
  },
  "fade-up": { bad: "<motion.div initial={{ opacity: 0, y: 20 }} whileInView={{ opacity: 1, y: 0 }} />", good: "<div class=\"row\" />" },
  "count-up": { bad: "<CountUp end={10000} />", good: "<p>{count}</p>" },
  "bounce-easing": {
    bad: ".pop { transition: transform 300ms cubic-bezier(.34, 1.56, .64, 1); }",
    good: ".pop { transition: transform 160ms cubic-bezier(0.25, 1, 0.5, 1); }",
  },
  "ambient-loop": { bad: ".blob { animation: drift 8s ease-in-out infinite; }", good: ".row { animation: pulse 600ms steps(4) 2; }" },
  "hover-lift": { bad: ".card:hover { transform: translateY(-2px); }", good: ".card:hover { color: var(--ink); }" },
  aspirational: { bad: "<h1>Elevate your workflow</h1>", good: "<h1>See which session needs you</h1>" },
  buzzwords: { bad: "<p>A seamless, cutting-edge experience</p>", good: "<p>Shows every session on this machine</p>" },
  tricolon: { bad: "<p>Faster, smarter, simpler.</p>", good: "<p>Faster builds.</p>" },
  "arrow-links": { bad: '<a href="/a">Docs →</a><a href="/b">Pricing →</a>', good: '<a href="/a">Docs →</a><a href="/b">Pricing</a>' },
  "generic-cta": { bad: "<button>Get started</button>", good: "<button>Install the hooks</button>" },
  "em-dash": { bad: "<p>Fast — safe — simple — done</p>", good: "<p>Fast, safe — and done</p>" },
  "full-stop-heading": { bad: "<h1>Every session, one window.</h1>", good: "<h1>Every session, one window</h1>" },
  "placeholder-copy": { bad: "<p>Lorem ipsum dolor sit amet</p>", good: "<p>Waiting for your answer</p>" },
  "colour-literal": {
    bad: ".title { color: #EDE9F2; }",
    good: ":root {\n  --text: #EDE9F2;\n}\n.title { color: var(--text); }\n<a href=\"#faded\">Top</a>",
  },
  "z-index-max": { bad: ".modal { z-index: 9999; }", good: ".modal { z-index: 10; }" },
  "important-pile": {
    bad: ".a { color: red !important; margin: 0 !important; padding: 0 !important; }",
    good: ".sr-only { position: absolute !important; }",
  },
  "generator-tag": { bad: '<meta name="generator" content="v0 by Vercel">', good: '<meta name="description" content="Peep This">' },
  "placeholder-media": { bad: '<img src="https://i.pravatar.cc/150?u=1">', good: '<img src="./shots/row.png">' },
  "arbitrary-px": { bad: '<div class="p-[13px] gap-[7px] mt-[22px]">', good: '<div class="p-4 gap-2 mt-[22px]">' },
};

describe("ai-tells grep rules", () => {
  it("has unique ids and a fixture pair for every rule", () => {
    const ids = RULES.map((r) => r.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(Object.keys(FIXTURES).sort()).toEqual([...ids].sort());
  });

  it.each(RULES.map((r) => [r.id, r]))("%s hits the bad snippet", (id, rule) => {
    expect(scan(FIXTURES[id].bad, [rule]).length).toBeGreaterThan(0);
  });

  it.each(RULES.map((r) => [r.id]))("%s good snippet hits no rule", (id) => {
    expect(scan(FIXTURES[id].good)).toEqual([]);
  });

  it("matches the grep entries in ai-tells.md, both ways", () => {
    const md = fs.readFileSync(new URL("../ai-tells.md", import.meta.url), "utf8");
    const entries = [...md.matchAll(/^- \*\*([\w-]+)\*\* · (P[012]) · (grep|eye) ·/gm)];
    const grep = Object.fromEntries(entries.filter((m) => m[3] === "grep").map((m) => [m[1], m[2]]));
    expect(Object.keys(grep).sort()).toEqual(RULES.map((r) => r.id).sort());
    for (const r of RULES) expect(grep[r.id], r.id).toBe(r.severity);
    const all = entries.map((m) => m[1]);
    expect(new Set(all).size).toBe(all.length);
  });
});

describe("scan", () => {
  it("reports the 1-based line of each hit", () => {
    expect(scan("a {}\n.b { z-index: 9999; }", RULES.filter((r) => r.id === "z-index-max"))).toEqual([{ id: "z-index-max", line: 2 }]);
  });

  it("honours scrub-ignore with a reason on the same line or the line above", () => {
    const same = ".m { z-index: 9999; } /* scrub-ignore: z-index-max over the native title bar */";
    const above = "/* scrub-ignore: z-index-max over the native title bar */\n.m { z-index: 9999; }";
    expect(scan(same)).toEqual([]);
    expect(scan(above)).toEqual([]);
  });

  it("ignores a scrub-ignore with no reason or another id", () => {
    expect(scan(".m { z-index: 9999; } /* scrub-ignore: z-index-max */").map((f) => f.id)).toEqual(["z-index-max"]);
    expect(scan(".m { z-index: 9999; } /* scrub-ignore: glass not this one */").map((f) => f.id)).toEqual(["z-index-max"]);
  });

  it("applies min after ignores", () => {
    const text = "<p>a — b — c</p>\n<p>d — e</p> <!-- scrub-ignore: em-dash quoted copy -->";
    expect(scan(text)).toEqual([]);
  });
});
