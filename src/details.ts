import type { Session } from "./model";
import { contextFill } from "./rowView";
import { STATE_LABEL } from "./stateMeta";

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

/** The open row's facts: tags (time in state, model, kind), last tool, context and cwd. */
export type OpenFacts = {
  since: string;
  model: string;
  kind: string;
  /** Full text; CSS clamps it to two lines with an end ellipsis. */
  tool: string;
  ctx: number | null;
  cwd: string;
};

const DASH = "—";

export function openFacts(s: Session, now: number): OpenFacts {
  const tool = s.lastTool ? [s.lastTool.name, s.lastTool.target].filter(Boolean).join(" ") : "";
  return {
    since: `${STATE_LABEL[s.state] ?? s.state} for ${formatDuration(now - s.stateSince)}`,
    model: s.model ?? "",
    kind: [s.kind, s.entrypoint].filter(Boolean).join(" · "),
    tool: tool || DASH,
    ctx: contextFill(s.contextPct),
    cwd: s.cwd,
  };
}
