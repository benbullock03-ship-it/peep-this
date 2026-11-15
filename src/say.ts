import { byOldest, questionOf, SHELL_TOOLS, targetOf } from "./approval";
import type { Agent, LastTool, PendingApproval, Session } from "./model";

/** Mirrors `STUCK_AFTER_MS` in state.rs: a session turns stuck this long after its last event. */
export const STUCK_AFTER_MS = 6 * 60 * 1000;

// Placeholder tasks state.rs writes when it has nothing better to say.
const DEFAULT_TASKS = new Set(["", "Idle", "Thinking", "Working", "Waiting for input", "Done", "Error"]);

const meaningful = (task: string) => !DEFAULT_TASKS.has(task.trim());

// Short verbs whose -ing form doubles the last consonant.
const DOUBLED = new Set(["run", "rerun", "get", "set", "reset", "stop", "put", "cut", "plan", "ship", "drop", "grep", "commit", "submit", "chop", "tag", "dig", "skip", "strip", "wrap", "zip", "unzip", "swap", "map", "log", "pin", "spin", "scan"]);

function gerund(word: string): string {
  const parts = word.split("-");
  const last = parts.pop()!;
  const lower = last.toLowerCase();
  let ing: string;
  if (!/^[a-z]+$/i.test(last) || lower.endsWith("ing")) ing = last;
  else if (DOUBLED.has(lower)) ing = last + last.slice(-1) + "ing";
  else if (lower.endsWith("ie")) ing = last.slice(0, -2) + "ying";
  else if (lower.endsWith("e") && !/(ee|ye|oe)$/.test(lower) && lower.length > 2) ing = last.slice(0, -1) + "ing";
  else ing = last + "ing";
  return [...parts, ing].join("-");
}

/** "Run the test suite" -> "Running the test suite". */
export function describeAsDoing(description: string): string {
  const d = description.trim().replace(/[.…]+$/, "");
  const m = /^(\S+)(.*)$/.exec(d);
  if (!m) return d;
  const g = gerund(m[1]);
  return g.charAt(0).toUpperCase() + g.slice(1) + m[2];
}

const firstLower = (s: string) => s.charAt(0).toLowerCase() + s.slice(1);

function tokens(segment: string): string[] {
  const out: string[] = [];
  const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
  for (let m; (m = re.exec(segment)); ) out.push(m[1] ?? m[2] ?? m[3]);
  return out;
}

/** Splits at unquoted `|`, `&`, `;`, newlines and redirects; everything after the first redirect is dropped. */
function segments(command: string): string[] {
  const out: string[] = [];
  let cur = "";
  let quote = "";
  for (const ch of command) {
    if (quote) {
      if (ch === quote) quote = "";
      cur += ch;
    } else if (ch === '"' || ch === "'") {
      quote = ch;
      cur += ch;
    } else if (ch === "<" || ch === ">") {
      out.push(cur);
      return out;
    } else if (ch === "|" || ch === "&" || ch === ";" || ch === "\n") {
      out.push(cur);
      cur = "";
    } else cur += ch;
  }
  out.push(cur);
  return out;
}

const basename = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? p;
const PREFIXES = new Set(["sudo", "env", "time", "nohup", "exec", "command", "&"]);
const ARG_MAX = 24;

/** Program basename plus its first plain argument: `.venv/Scripts/python - <<'EOF'` -> "python". */
export function shortCommand(command: string): string {
  const segs = segments(command).map(tokens).filter((t) => t.length > 0);
  const useful = segs.filter((t) => !/^(cd|pushd|set-location|sl)$/i.test(t[0]));
  for (const toks of useful.length ? useful : segs) {
    let i = 0;
    while (i < toks.length && (/^[A-Za-z_][A-Za-z0-9_]*=/.test(toks[i]) || PREFIXES.has(toks[i]))) i++;
    if (i >= toks.length) continue;
    const prog = basename(toks[i]).replace(/\.(exe|cmd|bat|ps1)$/i, "");
    const arg = toks[i + 1];
    if (!arg) return prog;
    // `npm run build`, `python -m pytest`: the first argument alone says nothing.
    if (RUNNER_ARGS.has(arg) && plainArg(toks[i + 2])) return `${prog} ${arg} ${plainArg(toks[i + 2])}`;
    const shown = plainArg(arg);
    return shown ? `${prog} ${shown}` : prog;
  }
  return "";
}

const RUNNER_ARGS = new Set(["run", "exec", "x", "dlx", "-m"]);

function plainArg(arg: string | undefined): string {
  if (!arg || arg.startsWith("-") || /\s/.test(arg) || arg.includes("$")) return "";
  const shown = /[\\/]/.test(arg) ? basename(arg) : arg;
  return shown.length <= ARG_MAX ? shown : "";
}

/** The bubble's name for a shell call: its description as a doing phrase, else the short command. */
function shellDoing(description: string | undefined, command: string): string {
  if (description?.trim()) return describeAsDoing(description);
  const short = shortCommand(command);
  return short ? `Running ${short}` : "Running a command";
}

function runningLine(t: LastTool | undefined, task: string): string {
  if (t && SHELL_TOOLS.has(t.name)) return `${shellDoing(t.description, t.target ?? "")}…`;
  const tool = t ? [t.name, t.target].filter(Boolean).join(" ") : meaningful(task) ? task : "";
  return tool ? `Running ${tool}…` : "Working…";
}

/** The classifier's flagged span of the command, so `git fetch && git reset --hard` asks about the reset. */
function riskyPart(p: PendingApproval): string {
  const { field, span } = p.risk;
  const text = field ? p.input[field] : undefined;
  if (!span || typeof text !== "string") return "";
  return new TextDecoder().decode(new TextEncoder().encode(text).slice(span[0], span[1]));
}

function askFor(p: PendingApproval): string {
  const q = questionOf(p);
  if (q) return q.text.split("\n")[0] || "I have a question for you.";
  if (SHELL_TOOLS.has(p.tool)) {
    const desc = typeof p.input.description === "string" ? p.input.description.trim().replace(/[.…]+$/, "") : "";
    if (desc) return `Can I ${firstLower(desc)}?`;
    const short = shortCommand(riskyPart(p) || String(p.input.command ?? ""));
    return short ? `Can I run ${short}?` : `Can I use ${p.tool}?`;
  }
  const target = targetOf(p);
  return target ? `Can I use ${p.tool} on ${target}?` : `Can I use ${p.tool}?`;
}

/** The waiting line: a held request first, else the notification text with any shell command shortened. */
function waitingLine(task: string, t: LastTool | undefined, pending: readonly PendingApproval[] | undefined): string {
  if (pending?.length) return askFor([...pending].sort(byOldest)[0]);
  if (task.startsWith("Needs permission: ") && t && SHELL_TOOLS.has(t.name)) {
    const desc = t.description?.trim().replace(/[.…]+$/, "");
    if (desc) return `Can I ${firstLower(desc)}?`;
    const short = shortCommand(t.target ?? "");
    return short ? `Can I run ${short}?` : `Can I use ${t.name}?`;
  }
  return meaningful(task) ? task : "Waiting for you.";
}

const stuckLine = (since: number, now: number) => `zzz… no reply for ${Math.floor((STUCK_AFTER_MS + Math.max(0, now - since)) / 60_000)} min`;

/** The speech-bubble line for a session, from its state and what the snapshot knows. */
export function sayFor(s: Session, now: number): string {
  if (!s.alive) return "Session closed. Bye!";
  const task = s.task ?? "";
  switch (s.state) {
    case "working":
      return runningLine(s.lastTool, task);
    case "waiting":
      return waitingLine(task, s.lastTool, s.pending);
    case "error":
      return meaningful(task) ? task : "Something went wrong.";
    case "stuck":
      return stuckLine(s.stateSince, now);
    case "success":
      return "All done!";
    case "thinking":
      return meaningful(task) ? task : "Hmm… thinking it over.";
    default:
      return "Resting. Ready when you are.";
  }
}

/** The bubble line for a subagent child row; `pending` is that agent's held requests only. */
export function kidSayFor(a: Agent, pending: readonly PendingApproval[] | undefined, now: number): string {
  const task = a.task ?? "";
  const asked = a.description.trim();
  if (a.endedAt !== undefined) {
    if (a.state === "idle") return "Stopped when the session closed.";
    if (a.result) return a.result;
    if (a.state === "error") return meaningful(task) ? task : "Something went wrong.";
    return "All done!";
  }
  switch (a.state) {
    case "working":
      return runningLine(a.lastTool, task);
    case "waiting":
      return waitingLine(task, a.lastTool, pending);
    case "error":
      return meaningful(task) ? task : "Something went wrong.";
    case "stuck":
      return stuckLine(a.stateSince, now);
    case "thinking":
      return meaningful(task) ? task : asked || "Hmm… thinking it over.";
    default:
      return asked || "Resting.";
  }
}
