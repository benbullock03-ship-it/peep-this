// Mirrors the serialised Rust types in src-tauri/src/state.rs and store.rs (spec §5.3).

export const STATES = ["idle", "thinking", "working", "waiting", "success", "error", "stuck"] as const;
export type State = (typeof STATES)[number];

export type LastTool = { name: string; target?: string };

export type Session = {
  id: string;
  pid: number;
  name: string;
  cwd: string;
  kind: string;
  entrypoint: string;
  model?: string;
  state: State;
  stateSince: number;
  task: string;
  lastTool?: LastTool;
  subagents: number;
  contextPct?: number;
  alive: boolean;
  endedAt?: number;
};

export type RateLimits = { fiveHourPct: number; sevenDayPct: number; at: number };

export type Snapshot = {
  sessions: Session[];
  rateLimits?: RateLimits;
  integration: "installed" | "missing" | "outdated";
};
