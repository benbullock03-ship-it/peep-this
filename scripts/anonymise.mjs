// Free text that can carry the user's code or words; only the payload's shape matters to the parsers.
const REDACT_KEYS = new Set([
  "prompt",
  "content",
  "new_string",
  "old_string",
  "last_assistant_message",
  "stdout",
  "stderr",
  "tool_response",
]);

const UUID = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi;

const slug = (p) => p.replace(/[^A-Za-z0-9]/g, "-");
const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** Path spellings Claude Code and Git Bash use for one Windows path. */
function spellings(win) {
  const fwd = win.replace(/\\/g, "/");
  const out = [win, fwd, slug(win)];
  const drive = fwd.match(/^([A-Za-z]):(\/.*)?$/);
  if (drive) out.push(`/${drive[1].toLowerCase()}${drive[2] ?? ""}`);
  return out;
}

/**
 * Returns a function that deep-copies a JSON value with paths, names, ids and free text replaced.
 * `cwds` are project folders; `names` are session names. One anonymiser keeps ids stable across files.
 */
export function makeAnonymiser({ home, user, host, cwds = [], names = [] }) {
  const pairs = [];
  cwds.forEach((cwd, i) => {
    const fake = `C:\\work\\demo-${i + 1}`;
    const [a, b, c, d] = spellings(cwd);
    pairs.push([a, fake], [b, fake.replace(/\\/g, "/")], [c, slug(fake)]);
    if (d) pairs.push([d, `/c/work/demo-${i + 1}`]);
    const base = cwd.split(/[\\/]/).pop();
    if (base) pairs.push([base, `demo-${i + 1}`], [slug(base), `demo-${i + 1}`]);
  });
  names.forEach((n, i) => pairs.push([n, `session-${i + 1}`]));
  if (home) {
    const [a, b, c, d] = spellings(home);
    pairs.push([a, "C:\\Users\\user"], [b, "C:/Users/user"], [c, "C--Users-user"]);
    if (d) pairs.push([d, "/c/Users/user"]);
  }
  if (user) pairs.push([user, "user"]);
  if (host) pairs.push([host, "test-host"]);
  // Longest first, so a cwd inside home is replaced before home itself.
  const rules = pairs
    .filter(([from]) => from)
    .sort((x, y) => y[0].length - x[0].length)
    .map(([from, to]) => [new RegExp(escapeRe(from), "gi"), to]);

  const ids = new Map();
  const fakeId = (id) => {
    const key = id.toLowerCase();
    if (!ids.has(key)) ids.set(key, `00000000-0000-4000-8000-${String(ids.size + 1).padStart(12, "0")}`);
    return ids.get(key);
  };

  const text = (s) => rules.reduce((acc, [re, to]) => acc.replace(re, to), s).replace(UUID, fakeId);

  const walk = (v, redact) => {
    if (typeof v === "string") return redact ? "<redacted>" : text(v);
    if (Array.isArray(v)) return v.map((x) => walk(x, redact));
    if (v && typeof v === "object") {
      return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, walk(x, redact || REDACT_KEYS.has(k))]));
    }
    return v;
  };
  return (value) => walk(value, false);
}
