import type { PendingApproval } from "./model";

export type DiffLine = { kind: "same" | "add" | "del"; text: string };
export type Diff = { lines: DiffLine[]; hidden: number };

/** Above this many cells the LCS table is skipped and the change shows as delete-all then add-all. */
const LCS_CELLS = 250_000;
export const DIFF_MAX_LINES = 200;

const splitLines = (s: string) => (s === "" ? [] : s.replace(/\r\n/g, "\n").split("\n"));

export function lineDiff(oldText: string, newText: string): DiffLine[] {
  const a = splitLines(oldText);
  const b = splitLines(newText);
  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start++;
  let endA = a.length;
  let endB = b.length;
  while (endA > start && endB > start && a[endA - 1] === b[endB - 1]) {
    endA--;
    endB--;
  }
  const out: DiffLine[] = a.slice(0, start).map((text) => ({ kind: "same", text }));
  const midA = a.slice(start, endA);
  const midB = b.slice(start, endB);
  const n = midA.length;
  const m = midB.length;
  if (n * m > LCS_CELLS) {
    for (const text of midA) out.push({ kind: "del", text });
    for (const text of midB) out.push({ kind: "add", text });
  } else {
    const lcs = Array.from({ length: n + 1 }, () => new Uint32Array(m + 1));
    for (let i = n - 1; i >= 0; i--)
      for (let j = m - 1; j >= 0; j--)
        lcs[i][j] = midA[i] === midB[j] ? lcs[i + 1][j + 1] + 1 : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
    let i = 0;
    let j = 0;
    while (i < n || j < m) {
      if (i < n && j < m && midA[i] === midB[j]) {
        out.push({ kind: "same", text: midA[i++] });
        j++;
      } else if (i < n && (j === m || lcs[i + 1][j] >= lcs[i][j + 1])) {
        out.push({ kind: "del", text: midA[i++] });
      } else {
        out.push({ kind: "add", text: midB[j++] });
      }
    }
  }
  for (const text of a.slice(endA)) out.push({ kind: "same", text });
  return out;
}

const str = (v: unknown) => (typeof v === "string" ? v : "");

/** The diff a file tool would apply, or null for tools that are not file edits. */
export function diffFor(tool: string, input: Record<string, unknown>): Diff | null {
  let lines: DiffLine[];
  if (tool === "Edit") lines = lineDiff(str(input.old_string), str(input.new_string));
  else if (tool === "MultiEdit" && Array.isArray(input.edits))
    lines = input.edits.flatMap((e: Record<string, unknown>, k) => [
      ...(k > 0 ? [{ kind: "same" as const, text: "…" }] : []),
      ...lineDiff(str(e?.old_string), str(e?.new_string)),
    ]);
  else if (tool === "Write") lines = lineDiff("", str(input.content));
  else if (tool === "NotebookEdit") lines = lineDiff("", str(input.new_source));
  else return null;
  const hidden = Math.max(0, lines.length - DIFF_MAX_LINES);
  return { lines: hidden ? lines.slice(0, DIFF_MAX_LINES) : lines, hidden };
}

export const SHELL_TOOLS = new Set(["Bash", "PowerShell"]);

const ARG_MAX = 160;
const ARGS_MAX = 6;

export function formatArgs(input: Record<string, unknown>): { key: string; value: string }[] {
  return Object.entries(input)
    .filter(([, v]) => v !== undefined && v !== null && v !== "")
    .slice(0, ARGS_MAX)
    .map(([key, v]) => {
      const s = typeof v === "string" ? v : JSON.stringify(v);
      return { key, value: s.length > ARG_MAX ? `${s.slice(0, ARG_MAX - 1)}…` : s };
    });
}

/** Converts a UTF-8 byte range (risk.rs spans) to UTF-16 indexes; null if it does not land on characters. */
export function byteSpanToChars(s: string, [from, to]: [number, number]): [number, number] | null {
  const enc = new TextEncoder();
  let bytes = 0;
  let start = -1;
  let i = 0;
  for (const ch of s) {
    if (bytes === from) start = i;
    if (bytes === to) return start >= 0 && to > from ? [start, i] : null;
    bytes += enc.encode(ch).length;
    i += ch.length;
  }
  if (bytes === to && start >= 0 && to > from) return [start, i];
  return null;
}

/** Splits `text` around the risky span so the card can mark it. */
export function riskParts(p: PendingApproval, field: string): { before: string; risky: string; after: string } | null {
  const text = p.input[field];
  if (typeof text !== "string" || p.risk.field !== field || !p.risk.span) return null;
  const r = byteSpanToChars(text, p.risk.span);
  if (!r) return null;
  return { before: text.slice(0, r[0]), risky: text.slice(r[0], r[1]), after: text.slice(r[1]) };
}

const norm = (p: string) => p.replace(/\\/g, "/").replace(/\/+$/, "");

/** Path relative to cwd when it sits inside it, else the path as given. */
export function displayPath(path: string, cwd: string): string {
  const p = norm(path);
  const c = norm(cwd);
  if (c && p.toLowerCase().startsWith(`${c.toLowerCase()}/`)) return p.slice(c.length + 1);
  return path;
}

export function targetOf(p: PendingApproval): string {
  const path = str(p.input.file_path) || str(p.input.notebook_path);
  if (path) return displayPath(path, p.cwd);
  return str(p.input.url) || str(p.input.pattern) || str(p.input.query);
}

export type Question = { text: string; options: { label: string; description: string }[]; count: number };

/** The first question of an `AskUserQuestion` request, or null for anything the widget can decide. */
export function questionOf(p: PendingApproval): Question | null {
  if (p.risk.tier !== "question") return null;
  const all = Array.isArray(p.input.questions) ? (p.input.questions as Record<string, unknown>[]) : [];
  const options = Array.isArray(all[0]?.options) ? (all[0].options as Record<string, unknown>[]) : [];
  return {
    text: str(all[0]?.question).trim(),
    options: options.map((o) => ({ label: str(o?.label), description: str(o?.description) })).filter((o) => o.label),
    count: all.length,
  };
}

export const byOldest =(a: PendingApproval, b: PendingApproval) => a.receivedAt - b.receivedAt || a.id.localeCompare(b.id);

export const REASONS_KEPT = 3;

/** Most recent first, deduplicated, capped. */
export function pushReason(list: readonly string[], reason: string): string[] {
  const r = reason.trim();
  if (!r) return list.slice(0, REASONS_KEPT);
  return [r, ...list.filter((x) => x !== r)].slice(0, REASONS_KEPT);
}

export const HOLD_MS = 1000;

/** Press-and-hold guard: `done` fires once the press lasts `ms`; release or cancel before that does nothing. */
export function createHold(done: () => void, ms = HOLD_MS) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return {
    start() {
      if (timer !== undefined) return;
      timer = setTimeout(() => {
        timer = undefined;
        done();
      }, ms);
    },
    cancel() {
      clearTimeout(timer);
      timer = undefined;
    },
    get active() {
      return timer !== undefined;
    },
  };
}
