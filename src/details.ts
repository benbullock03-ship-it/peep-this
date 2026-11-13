import type { Session } from "./model";

/** Fits the value column of a 360px widget at 11px mono. */
export const DETAIL_MAX_CHARS = 38;

/** Keeps both ends, so a path's drive and filename both stay visible. */
export function truncateMiddle(s: string, max = DETAIL_MAX_CHARS): string {
  const chars = Array.from(s);
  if (chars.length <= max) return s;
  const keep = max - 1;
  const head = Math.ceil(keep / 2);
  return chars.slice(0, head).join("") + "…" + chars.slice(chars.length - (keep - head)).join("");
}

export function formatDuration(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  if (h > 0) return `${h}h ${pad(m)}m`;
  if (m > 0) return `${m}m ${pad(s)}s`;
  return `${s}s`;
}

export type DetailField = { key: string; label: string; value: string; full: string };

const DASH = "—";

export function detailFields(s: Session, now: number): DetailField[] {
  const tool = s.lastTool ? [s.lastTool.name, s.lastTool.target].filter(Boolean).join(" ") : "";
  const ctx = s.contextPct === undefined || !Number.isFinite(s.contextPct) ? "" : `${Math.round(s.contextPct)}% used`;
  const kind = [s.kind, s.entrypoint].filter(Boolean).join(" · ");
  const f = (key: string, label: string, raw: string): DetailField => ({
    key,
    label,
    value: raw ? truncateMiddle(raw) : DASH,
    full: raw,
  });
  return [
    f("cwd", "cwd", s.cwd),
    f("model", "model", s.model ?? ""),
    f("tool", "last tool", tool),
    f("since", "in state", formatDuration(now - s.stateSince)),
    f("ctx", "context", ctx),
    f("kind", "kind", kind),
  ];
}
