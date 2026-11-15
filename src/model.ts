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
  /** Running Peep skill without the `peep:` namespace, e.g. "ticket"; cleared on Stop. */
  skill?: string;
  /** Linked ticket; `title` is absent when the ticket file was not found. */
  ticket?: { id: string; title?: string };
  contextPct?: number;
  /** mood.rs: at most one, still set during attention states (the bubble hides it there). */
  mood?: Mood;
  /** Absolute path of the clashed file while `mood` is startled. */
  clash?: string;
  alive: boolean;
  endedAt?: number;
  pending?: PendingApproval[];
};

export type Mood = "startled" | "cheer" | "greet" | "worry" | "yawn" | "sweat" | "nap";

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

/** `question` is an `AskUserQuestion` shown read-only (AV-50); it never reaches the trail or a rule. */
export type RiskTier = "question" | "safe" | "shell" | "destructive";

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

/** src-tauri/src/rules.rs `Shape`: cwd is lowercased with forward slashes. */
export type RuleShape = { cwd: string; tool: string; pattern: string };
export type Rule = RuleShape & { id: string; created: number; uses: number; lastUsed?: number };
export type RuleOffer = RuleShape & { id: string; sessionId: string; tier: RiskTier; at: number };
export type RulesView = { rules: Rule[]; offers: RuleOffer[]; error?: string };
export type RuleAnswer = "yes" | "not-now" | "never";

/** src-tauri/src/away.rs `Digest`: entries arrive grouped and ordered. */
export type DigestGroup = "blocked" | "failed" | "finished" | "running";
export type DigestEntry = {
  sessionId: string;
  name: string;
  group: DigestGroup;
  state?: State;
  alive: boolean;
  pending: boolean;
  tookMs: number;
  files: string[];
  fileCount: number;
  commands: number;
  released: number;
  lastLine?: string;
  partial: boolean;
  /** AV-33: one model sentence, arrives after the card when the setting is on. */
  summary?: string;
};
export type Digest = { since: number; until: number; entries: DigestEntry[] };

export type RateLimits ={ fiveHourPct: number; sevenDayPct: number; at: number };

/** Ticket file status (formats.md §2); `closed` never reaches a shelf. */
export type TicketStatus = "open" | "doing" | "blocked" | "deferred" | "done";
/** `title` is "" when the file has none; `reason`/`wake` only on parked tickets. */
export type RepoTicket = { id: string; title: string; status: TicketStatus; reason?: string; wake?: string };

/** src-tauri/src/tickets.rs `Repo`: one per repo root with live sessions; `sessions` are session ids. */
export type Repo = {
  root: string;
  sessions: string[];
  tickets: RepoTicket[];
  /** Latest ticket seen closing since the widget first read the repo; `at` is ms. */
  lastClosed?: { id: string; at: number };
};

export type Snapshot = {
  sessions: Session[];
  rateLimits?: RateLimits;
  integration: "installed" | "missing" | "outdated";
  /** Always sent by the backend; optional so the browser demo snapshot needs none. */
  repos?: Repo[];
};
