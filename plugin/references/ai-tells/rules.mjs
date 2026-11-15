// The grep half of ai-tells.md. Patterns marked "aad" are adapted from avoid-ai-design's
// scripts/detect.mjs (MIT, Copyright (c) 2026 ungspirit); see ai-tells.md "Sources".

/** @typedef {{ id: string, group: string, severity: "P0"|"P1"|"P2", re: RegExp, min?: number }} Rule */

/** @type {Rule[]} */
export const RULES = [
  // Colour
  { id: "purple-gradient", group: "Colour", severity: "P0", re: /(?:linear|radial|conic)-gradient\([^;{}]*?(?:indigo|violet|purple|blueviolet|#(?:6366f1|4f46e5|7c3aed|8b5cf6|a855f7|9333ea|6d28d9|667eea|764ba2)\b)|\b(?:from|via|to)-(?:indigo|violet|purple|fuchsia)-\d{3}\b/gi },
  { id: "violet-accent", group: "Colour", severity: "P1", re: /\b(?:bg|text|border|ring)-(?:indigo|violet)-[4-7]00\b/g },
  { id: "gradient-text", group: "Colour", severity: "P0", re: /(?:-webkit-)?background-clip\s*:\s*text\b|\bbg-clip-text\b/g },
  { id: "colour-glow", group: "Colour", severity: "P2", re: /box-shadow\s*:\s*0(?:px)?\s+0(?:px)?\s+\d{2,}px[^;}]*(?:#[0-9a-f]{3,8}\b|rgba?\((?!\s*0\s*,\s*0\s*,\s*0\b))|\bshadow-(?:indigo|violet|purple|pink|fuchsia|blue|sky|cyan|emerald|teal)-\d{3}\b/gi },
  { id: "cream-terracotta", group: "Colour", severity: "P1", re: /#(?:d97757|c96442|cc785c|f4f1ea)\b/gi },
  { id: "repeating-stripes", group: "Colour", severity: "P2", re: /repeating-(?:linear|radial)-gradient\(/g },

  // Type
  { id: "default-face", group: "Type", severity: "P0", re: /font-family\s*:\s*["']?(?:inter|geist)["']?\s*[,;}]|fonts\.googleapis\.com\/css2?\?family=Inter\b/gi },
  { id: "tasteful-face", group: "Type", severity: "P1", re: /(?:font-family\s*:[^;{}]*?|family=)["']?(?:space[\s+_]grotesk|instrument[\s+_]serif|fraunces|syne|sora|cal[\s+_]sans)\b/gi },
  { id: "tracked-caps", group: "Type", severity: "P1", re: /text-transform\s*:\s*uppercase[^{}]*?letter-spacing\s*:\s*0?\.\d|letter-spacing\s*:\s*0?\.\d[^{}]*?text-transform\s*:\s*uppercase|\buppercase\b[^"'`\n]*\btracking-(?:wide|wider|widest)\b|\btracking-(?:wide|wider|widest)\b[^"'`\n]*\buppercase\b/g },
  { id: "neg-tracking", group: "Type", severity: "P2", re: /letter-spacing\s*:\s*-|\btracking-(?:tight|tighter)\b/g },
  { id: "accent-word", group: "Type", severity: "P1", re: /<h[12]\b[^>]*>[^<]+<(?:em|i)\b/gi },
  { id: "eyebrow", group: "Type", severity: "P1", re: /class(?:Name)?=["'][^"']*\b(?:eyebrow|kicker|overline)\b/g },

  // Layout
  { id: "stat-strip", group: "Layout", severity: "P1", re: /\b\d+(?:\.\d+)?[KkMm]\+|\b99\.9+%|\b4\.\d\s?(?:★|stars?\b)/g },
  { id: "numbered-markers", group: "Layout", severity: "P1", re: />\s*0[1-9]\s*</g, min: 3 },
  { id: "default-shell", group: "Layout", severity: "P1", re: /\b(?:container|max-w-7xl)\b[^"'`\n]*\bmx-auto\b|\bmx-auto\b[^"'`\n]*\b(?:container|max-w-7xl)\b/g, min: 3 },
  { id: "most-popular", group: "Layout", severity: "P1", re: /\bmost popular\b/gi },

  // Components
  { id: "shadcn-radius", group: "Components", severity: "P1", re: /--radius\s*:\s*0\.(?:5|625)rem/g },
  { id: "card-kit", group: "Components", severity: "P1", re: /\brounded-(?:xl|2xl|3xl)\b[^"'`\n]*\bshadow-(?:md|lg|xl|2xl)\b|\bshadow-(?:md|lg|xl|2xl)\b[^"'`\n]*\brounded-(?:xl|2xl|3xl)\b/g },
  { id: "glass", group: "Components", severity: "P1", re: /backdrop-filter\s*:\s*blur|\bbackdrop-blur\b/g },
  { id: "side-stripe", group: "Components", severity: "P1", re: /border-(?:left|inline-start)\s*:\s*[2-9]px\s+solid|\bborder-l-(?:2|4|8)\b/g },
  { id: "headline-pill", group: "Components", severity: "P2", re: /class(?:Name)?=["'][^"']*\b(?:badge|pill|chip)\b[^"']*["'][^>]*>[^<]*<\/\w+>\s*<h1\b/g },
  { id: "pill-radius", group: "Components", severity: "P2", re: /border-radius\s*:\s*(?:999|9999)px|\brounded-full\b/g },
  { id: "effect-library", group: "Components", severity: "P1", re: /<(?:Spotlight|BackgroundBeams|AnimatedBeam|ShimmerButton|NumberTicker|BorderBeam|Meteors|SparklesCore|AuroraBackground|MagicCard|ShineBorder|Marquee)\b|components\/(?:magicui|aceternity)\b/g },
  { id: "emoji-icons", group: "Components", severity: "P1", re: /(?![©®™↔↕‼⁉])\p{Extended_Pictographic}/gu, min: 2 },
  { id: "worn-icons", group: "Components", severity: "P1", re: /import\s*\{[^}]*\b(?:Sparkles|Zap|Rocket)\b[^}]*\}\s*from\s*["']lucide-react["']/g },
  { id: "fake-window", group: "Components", severity: "P2", re: /#(?:ff5f57|febc2e|ffbd2e|28c840|27c93f)\b/gi },
  { id: "default-dark-card", group: "Components", severity: "P1", re: /\bbg-(?:zinc|gray|slate|neutral)-9(?:00|50)\b[^"'`\n]*\bborder-white\/(?:5|10)\b/g },
  { id: "hard-shadow", group: "Components", severity: "P2", re: /box-shadow\s*:\s*(?:-?(?:[4-9]|\d{2,})px\s+-?\d+(?:px)?|-?\d+(?:px)?\s+-?(?:[4-9]|\d{2,})px)\s+0(?:px)?(?=[\s;,}])/g },

  // Motion
  { id: "fade-up", group: "Motion", severity: "P2", re: /initial=\{\{\s*opacity\s*:\s*0\s*,\s*y\s*:|\bwhileInView=|data-aos=["']fade-up|@keyframes\s+(?:fade-?in-?up|fadeUp|slide-?up)\b/gi },
  { id: "count-up", group: "Motion", severity: "P2", re: /\bNumberTicker\b|\bCountUp\b|react-countup|\buseCountUp\b|\btweened\s*\(/g },
  { id: "bounce-easing", group: "Motion", severity: "P2", re: /cubic-bezier\(\s*[\d.]+\s*,\s*(?:-|1\.\d*[1-9]|[2-9])|cubic-bezier\([^,)]+,[^,)]+,[^,)]+,\s*(?:-|1\.\d*[1-9]|[2-9])|\banimate-bounce\b|\beaseOutBack\b|\beaseInOutBack\b/g },
  { id: "ambient-loop", group: "Motion", severity: "P2", re: /animation(?:-iteration-count)?\s*:[^;}]*\binfinite\b|\banimate-(?:pulse|ping)\b/g },
  { id: "hover-lift", group: "Motion", severity: "P2", re: /:hover[^{}]*\{[^}]*transform\s*:\s*translateY\(\s*-|\bhover:-translate-y-/g },

  // Copy
  { id: "aspirational", group: "Copy", severity: "P1", re: /\b(?:elevate|unleash|supercharge|revolutioni[sz]e|transform) your\b|\ball-in-one platform\b|\bthe future of\b/gi },
  { id: "buzzwords", group: "Copy", severity: "P2", re: /\b(?:seamless(?:ly)?|cutting-edge|best-in-class|game-?changing|effortless(?:ly)?|next-gen(?:eration)?|world-class)\b/gi },
  { id: "tricolon", group: "Copy", severity: "P2", re: /\b[A-Za-z]+(?:er|ly|est),\s+[a-z]+(?:er|ly|est),\s+(?:and\s+)?[a-z]+(?:er|ly|est)[.!]/g },
  { id: "arrow-links", group: "Copy", severity: "P1", re: /[^\s<>"'=]\s?[→⟶➜➔]\s*(?:<\/|["'`])/g, min: 2 },
  { id: "generic-cta", group: "Copy", severity: "P2", re: />\s*(?:Get started|Learn more|Click here|Submit)\s*</gi },
  { id: "em-dash", group: "Copy", severity: "P2", re: /—/g, min: 3 },
  { id: "full-stop-heading", group: "Copy", severity: "P2", re: /<h[1-3]\b[^>]*>[^<]{3,}[^.]\.\s*<\/h[1-3]>/gi },
  { id: "placeholder-copy", group: "Copy", severity: "P0", re: /\blorem ipsum\b|\bJohn Doe\b|\bAcme (?:Inc|Corp)\b/gi },

  // Code
  { id: "colour-literal", group: "Code", severity: "P2", re: /^(?![^\n]*--[\w-]+\s*:)[^\n]*?(?:[:(,]\s*|\s)(?:#[0-9a-fA-F]{3,8}\b|rgba?\(\s*\d)/gm },
  { id: "z-index-max", group: "Code", severity: "P2", re: /z-index\s*:\s*\d{4,}|\bz-\[\d{4,}\]/g },
  { id: "important-pile", group: "Code", severity: "P2", re: /!important\b/g, min: 3 },
  { id: "generator-tag", group: "Code", severity: "P1", re: /lovable-tagger|gpt-?engineer|v0\.dev|bolt\.new|name=["']generator["'][^>]*content=["'](?:v0|Lovable|Bolt)/gi },
  { id: "placeholder-media", group: "Code", severity: "P1", re: /dicebear|pravatar|boring-avatars|ui-avatars|robohash|randomuser\.me|placehold\.co|via\.placeholder|picsum\.photos/gi },
  { id: "arbitrary-px", group: "Code", severity: "P2", re: /\b-?(?:p|m|px|py|pt|pb|pl|pr|mx|my|mt|mb|ml|mr|gap|w|h|top|left|right|bottom|text)-\[\d+(?:\.\d+)?px\]/g, min: 3 },
];

// The reason must start with a letter, so a bare id before `*/` or `-->` waives nothing.
const IGNORE = /scrub-ignore:\s*([\w-]+)\s+[A-Za-z]/g;

/** Ids waived by a `scrub-ignore: ID reason` on each 1-based line. An ignore without a reason waives nothing. */
function ignoresByLine(lines) {
  return lines.map((l) => new Set([...l.matchAll(IGNORE)].map((m) => m[1])));
}

/** Findings for one file's text: `{ id, line }`, after inline ignores and each rule's `min`. */
export function scan(text, rules = RULES) {
  const lines = text.split("\n");
  const ignores = ignoresByLine(lines);
  const starts = [0];
  for (let i = 0; i < text.length; i++) if (text[i] === "\n") starts.push(i + 1);
  const lineOf = (idx) => {
    let lo = 0, hi = starts.length - 1;
    while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (starts[mid] <= idx) lo = mid; else hi = mid - 1; }
    return lo + 1;
  };
  const out = [];
  for (const rule of rules) {
    const hits = [...text.matchAll(rule.re)]
      .map((m) => lineOf(m.index))
      .filter((line) => !ignores[line - 1].has(rule.id) && !(line > 1 && ignores[line - 2].has(rule.id)));
    if (hits.length >= (rule.min ?? 1)) for (const line of hits) out.push({ id: rule.id, line });
  }
  return out;
}
