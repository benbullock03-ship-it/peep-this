import { displayPath } from "./approval";
import type { TrailEntry, TrailOutcome } from "./model";

export const ROW_TRAIL = 5;
/** The settings log renders at most this many matches; the count line says how many more. */
export const LOG_MAX = 300;

export const OUTCOMES: readonly TrailOutcome[] = ["allowed", "denied", "released", "auto-allowed"];

const MARKS: Record<TrailOutcome, string> = {
  allowed: "✓",
  "auto-allowed": "✓",
  denied: "✗",
  released: "–",
};

const LABELS: Record<TrailOutcome, string> = {
  allowed: "allowed",
  "auto-allowed": "auto-allowed",
  denied: "denied",
  released: "released to terminal",
};

const VIA: Record<string, string> = {
  "timed-out": "timed out",
  "terminal-focused": "terminal focused",
  "moved-on": "session moved on",
  "session-ended": "session ended",
  "answered-in-terminal": "answered in terminal",
  "widget-hidden": "widget hidden",
  "widget-quit": "widget quit",
};

export const markOf = (o: TrailOutcome) => MARKS[o] ?? "?";

/** Paths inside the session cwd show relative; commands as they were. */
export function shownTarget(e: TrailEntry): string {
  return e.target ? displayPath(e.target, e.cwd) : "";
}

const pad = (n: number) => String(n).padStart(2, "0");
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** `14:05:09` today, `3 Nov 14:05` otherwise (local time). */
export function formatWhen(t: number, now: number): string {
  const d = new Date(t);
  const n = new Date(now);
  const hm = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  const sameDay = d.getFullYear() === n.getFullYear() && d.getMonth() === n.getMonth() && d.getDate() === n.getDate();
  return sameDay ? `${hm}:${pad(d.getSeconds())}` : `${d.getDate()} ${MONTHS[d.getMonth()]} ${hm}`;
}

/** Hover text: time, outcome and why, the reason, then the full target. */
export function entryTitle(e: TrailEntry, now: number): string {
  let head = `${formatWhen(e.t, now)} · ${LABELS[e.outcome] ?? e.outcome}`;
  if (e.via) head += ` (${VIA[e.via] ?? e.via})`;
  if (e.rule) head += ` · rule ${e.rule}`;
  const lines = [head];
  if (e.reason) lines.push(`Reason: ${e.reason}`);
  lines.push([e.tool, e.target].filter(Boolean).join(" "));
  return lines.join("\n");
}

export type TrailFilter = { session: string; outcome: TrailOutcome | ""; text: string };

export function filterTrail(entries: readonly TrailEntry[], f: TrailFilter): TrailEntry[] {
  const q = f.text.trim().toLowerCase();
  return entries.filter(
    (e) =>
      (!f.session || e.sessionId === f.session) &&
      (!f.outcome || e.outcome === f.outcome) &&
      (!q || [e.session, e.tool, e.target, e.reason ?? "", e.cwd].some((s) => s.toLowerCase().includes(q))),
  );
}

/** One option per session id, most recently seen first, named by its latest recorded name. */
export function sessionOptions(entries: readonly TrailEntry[]): { id: string; name: string }[] {
  const seen = new Map<string, { id: string; name: string; t: number }>();
  for (const e of entries) {
    const cur = seen.get(e.sessionId);
    if (!cur || e.t > cur.t) seen.set(e.sessionId, { id: e.sessionId, name: e.session || e.sessionId.slice(0, 8), t: e.t });
  }
  return [...seen.values()].sort((a, b) => b.t - a.t).map(({ id, name }) => ({ id, name }));
}

/** Changes whenever the rendered marks would. */
export const trailKey = (entries: readonly TrailEntry[] | undefined) =>
  (entries ?? []).map((e) => `${e.t}:${e.outcome}`).join(",");
