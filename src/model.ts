// Mirrors the serialised Rust types in src-tauri/src/state.rs and store.rs (spec §5.3).

export const STATES = ["idle", "thinking", "working", "waiting", "success", "error", "stuck"] as const;
export type State = (typeof STATES)[number];

/** `description` is the shell tool input's own description (Bash/PowerShell only). */
export type LastTool = { name: string; target?: string; description?: string };

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
  /** Running count from the backend; the UI renders `agents` instead. */
  subagents: number;
  agents?: Agent[];
  shells?: Shell[];
  contextPct?: number;
  alive: boolean;
  endedAt?: number;
  pending?: PendingApproval[];
};

/** A subagent of a session (state.rs `Agent`); finished once `endedAt` is set. */
export type Agent = {
  id: string;
  agentType: string;
  description: string;
  state: State;
  stateSince: number;
  task: string;
  lastTool?: LastTool;
  startedAt: number;
  endedAt?: number;
  toolCount: number;
  result?: string;
};

export type ShellState = "running" | "quiet" | "serving" | "exited" | "failed";

/** A background shell (state.rs `Shell`); ended once `endedAt` is set. */
export type Shell = {
  id: string;
  command: string;
  description: string;
  state: ShellState;
  stateSince: number;
  startedAt: number;
  endedAt?: number;
  port?: number;
  exitCode?: number;
  lastLine?: string;
};

export type RiskTier = "safe" | "shell" | "destructive";

export type PendingApproval = {
  id: string;
  tool: string;
  input: Record<string, unknown>;
  cwd: string;
  agentId?: string;
  receivedAt: number;
  releaseAt: number;
  risk: { tier: RiskTier; field?: string; span?: [number, number]; reason: string };
};

export type TrailOutcome = "allowed" | "denied" | "released" | "auto-allowed";

/** One line of the decision trail (src-tauri/src/trail.rs). */
export type TrailEntry = {
  t: number;
  sessionId: string;
  session: string;
  cwd: string;
  tool: string;
  target: string;
  tier: RiskTier;
  outcome: TrailOutcome;
  reason?: string;
  rule?: string;
  via?: string;
  agentId?: string;
};

export type RateLimits ={ fiveHourPct: number; sevenDayPct: number; at: number };

export type Snapshot = {
  sessions: Session[];
  rateLimits?: RateLimits;
  integration: "installed" | "missing" | "outdated";
};
