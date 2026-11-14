import type { Identity } from "./identity";
import type { PendingApproval, Session, Snapshot, State } from "./model";

// The six mock agents from the "In a monitor" section of design/peep-this.dc.html.
const AGENTS: { name: string; identity: Identity; work: string; state: State }[] = [
  { name: "ingest-01", identity: { color: "yellow", accessory: "hardhat" }, work: "Parsing invoices batch 14", state: "working" },
  { name: "research-02", identity: { color: "lavender", accessory: "glasses" }, work: "Reading vendor docs", state: "thinking" },
  { name: "triage-03", identity: { color: "pink", accessory: "headphones" }, work: "Labelling support tickets", state: "waiting" },
  { name: "digest-04", identity: { color: "green", accessory: "chef" }, work: "Cooking the weekly digest", state: "success" },
  { name: "sync-05", identity: { color: "blue", accessory: "bow" }, work: "Syncing CRM records", state: "error" },
  { name: "scrape-06", identity: { color: "yellow", accessory: "party" }, work: "Crawling pricing pages", state: "stuck" },
];

const TASKS: Record<Exclude<State, "working">, string> = {
  idle: "Queued",
  thinking: "Planning next step",
  waiting: "Waiting for approval",
  success: "Finished",
  error: "Tool call failed",
  stuck: "No response for 6 min",
};

const FLOW: State[] = ["idle", "thinking", "working", "working", "thinking", "working", "waiting", "success", "error", "stuck"];

const HOLD_MS = 60_000;

function spanOf(text: string, token: string): [number, number] {
  const at = new TextEncoder().encode(text.slice(0, text.indexOf(token))).length;
  return [at, at + new TextEncoder().encode(token).length];
}

const SHELL_CMD = "npm run build && npx vitest run src/rows.test.ts";
const DESTRUCTIVE_CMD = "git fetch origin && git reset --hard origin/main && git push --force origin main";

// One card per tier, keyed by the demo session index that holds it.
const PENDING: Record<number, Omit<PendingApproval, "receivedAt" | "releaseAt">> = {
  2: {
    id: "demo-req-safe",
    tool: "Edit",
    cwd: "C:/demo/triage-03",
    input: {
      file_path: "C:/demo/triage-03/src/labels.ts",
      old_string: 'export const LABELS = ["bug", "question"];\nexport const DEFAULT = "question";',
      new_string: 'export const LABELS = ["bug", "question", "billing"];\nexport const DEFAULT = "question";\nexport const URGENT = ["billing"];',
    },
    risk: { tier: "safe", reason: "edit inside the session folder" },
  },
  0: {
    id: "demo-req-shell",
    tool: "Bash",
    cwd: "C:/Users/demo/Projects/accounts-payable/pipelines/ingest-01",
    input: { command: SHELL_CMD, description: "Build and run the row tests" },
    risk: { tier: "shell", field: "command", span: spanOf(SHELL_CMD, "npm run build"), reason: "shell command" },
  },
  5: {
    id: "demo-req-destructive",
    tool: "Bash",
    cwd: "C:/demo/scrape-06",
    input: { command: DESTRUCTIVE_CMD },
    risk: {
      tier: "destructive",
      field: "command",
      span: spanOf(DESTRUCTIVE_CMD, "git reset --hard"),
      reason: "git reset --hard discards local changes",
    },
  },
};

function arm(p: Omit<PendingApproval, "receivedAt" | "releaseAt">, now: number): PendingApproval {
  return { ...p, receivedAt: now, releaseAt: now + HOLD_MS };
}

const demoId = (i: number) => `demo-${i + 1}`;
const identities = new Map(AGENTS.map((a, i) => [demoId(i), a.identity]));

export function demoIdentity(s: Session): Identity {
  return identities.get(s.id) ?? { color: "yellow", accessory: "none" };
}

function taskFor(i: number, state: State): string {
  return state === "working" ? AGENTS[i].work : TASKS[state];
}

export function demoSnapshot(now: number): Snapshot {
  return {
    integration: "missing",
    rateLimits: { fiveHourPct: 42, sevenDayPct: 18, at: now },
    sessions: AGENTS.map((a, i) => withPending({
      id: demoId(i),
      pid: 1000 + i,
      name: a.name,
      cwd: i === 0 ? "C:/Users/demo/Projects/accounts-payable/pipelines/ingest-01" : `C:/demo/${a.name}`,
      kind: "interactive",
      entrypoint: i === 2 ? "claude-vscode" : "cli",
      model: i === 1 ? undefined : "claude-opus-5-5",
      lastTool: i === 0 ? { name: "Read", target: "pipelines/ingest/invoices/batch-14/normalise_line_items.py" } : undefined,
      state: a.state,
      stateSince: now - i * 60_000,
      task: taskFor(i, a.state),
      subagents: i === 0 ? 2 : 0,
      // Low, amber (70+) and red (90+) context for the segmented bars.
      contextPct: [74, 35, 93, 12, 47, undefined][i],
      alive: true,
    }, i, now)),
  };
}

function withPending(s: Session, i: number, now: number): Session {
  const p = PENDING[i];
  return p ? { ...s, state: "waiting", task: TASKS.waiting, pending: [arm(p, now)] } : s;
}

/** Drops a decided demo request; the session moves on as the backend would. */
export function demoDecide(snap: Snapshot, requestId: string, decision: "allow" | "deny", now: number): Snapshot {
  const sessions = snap.sessions.map((s, i) => {
    if (!s.pending?.some((p) => p.id === requestId)) return s;
    const pending = s.pending.filter((p) => p.id !== requestId);
    if (pending.length) return { ...s, pending };
    const state: State = decision === "allow" ? "working" : "thinking";
    return { ...s, pending: undefined, state, stateSince: now, task: taskFor(i, state) };
  });
  return { ...snap, sessions };
}

// Mirrors the design's simulation: one agent changes state per step.
export function demoStep(snap: Snapshot, now: number, rand: () => number = Math.random): Snapshot {
  // Held requests re-arm instead of releasing, so the cards stay up for a look.
  const sessions = snap.sessions.map((s) =>
    s.pending?.length ? { ...s, pending: s.pending.map((p) => (p.releaseAt <= now ? { ...p, receivedAt: now, releaseAt: now + HOLD_MS } : p)) } : s,
  );
  const free = sessions.flatMap((s, k) => (s.pending?.length ? [] : [k]));
  if (!free.length) return { ...snap, sessions, rateLimits: snap.rateLimits && { ...snap.rateLimits, at: now } };
  const i = free[Math.floor(rand() * free.length)];
  let next: State;
  do next = FLOW[Math.floor(rand() * FLOW.length)];
  while (next === sessions[i].state);
  sessions[i] = { ...sessions[i], state: next, stateSince: now, task: taskFor(i, next) };
  const rateLimits = snap.rateLimits && { ...snap.rateLimits, at: now };
  return { ...snap, sessions, rateLimits };
}
