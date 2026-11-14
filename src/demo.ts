import type { Identity } from "./identity";
import type { Session, Snapshot, State } from "./model";

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
    sessions: AGENTS.map((a, i) => ({
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
    })),
  };
}

// Mirrors the design's simulation: one agent changes state per step.
export function demoStep(snap: Snapshot, now: number, rand: () => number = Math.random): Snapshot {
  const sessions = snap.sessions.slice();
  const i = Math.floor(rand() * sessions.length);
  let next: State;
  do next = FLOW[Math.floor(rand() * FLOW.length)];
  while (next === sessions[i].state);
  sessions[i] = { ...sessions[i], state: next, stateSince: now, task: taskFor(i, next) };
  const rateLimits = snap.rateLimits && { ...snap.rateLimits, at: now };
  return { ...snap, sessions, rateLimits };
}
