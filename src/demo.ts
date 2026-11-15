import type { Identity } from "./identity";
import type { Agent, Digest, LastTool, PendingApproval, Repo, RepoTicket, Session, Shell, Snapshot, State } from "./model";

// The six mock agents from the "In a monitor" section of design/peep-this.dc.html.
const AGENTS: { name: string; identity: Identity; tool: LastTool & { target: string }; state: State }[] = [
  { name: "ingest-01", identity: { color: "yellow", accessory: "hardhat" }, tool: { name: "Read", target: "pipelines/ingest/invoices/batch-14/normalise_line_items.py" }, state: "working" },
  { name: "research-02", identity: { color: "lavender", accessory: "glasses" }, tool: { name: "WebFetch", target: "vendor docs" }, state: "thinking" },
  { name: "triage-03", identity: { color: "pink", accessory: "headphones" }, tool: { name: "Edit", target: "src/labels.ts" }, state: "waiting" },
  { name: "digest-04", identity: { color: "green", accessory: "chef" }, tool: { name: "Write", target: "digest/weekly.md" }, state: "success" },
  { name: "sync-05", identity: { color: "blue", accessory: "bow" }, tool: { name: "Bash", target: "npm run sync" }, state: "error" },
  { name: "scrape-06", identity: { color: "yellow", accessory: "party" }, tool: { name: "WebFetch", target: "pricing page" }, state: "stuck" },
];

// Shaped like the tasks state.rs writes, so the bubbles read as they would live.
const TASKS: Record<Exclude<State, "working" | "waiting">, string> = {
  idle: "Idle",
  thinking: "Planning the next step",
  success: "Done",
  error: "Build failed. A module is missing.",
  stuck: "No response for 6 min",
};

const asks = (tool: string) => `Claude needs your permission to use ${tool}`;

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
    agentId: "demo-agent-tests",
    input: { command: SHELL_CMD, description: "Build and run the row tests" },
    risk: { tier: "shell", field: "command", span: spanOf(SHELL_CMD, "npm run build"), reason: "shell command" },
  },
  1: {
    id: "demo-req-question",
    tool: "AskUserQuestion",
    cwd: "C:/demo/research-02",
    input: {
      questions: [
        {
          question: "The vendor API has two auth schemes. Which one should the client use for the nightly sync?",
          header: "Auth",
          options: [
            { label: "OAuth client credentials", description: "Rotating tokens; needs a client secret in the vault." },
            { label: "Static API key", description: "Simpler, but the key never expires." },
          ],
          multiSelect: false,
        },
        { question: "Should failed pages retry?", header: "Retry", options: [{ label: "Yes", description: "Up to 3 times." }, { label: "No", description: "Fail the run." }], multiSelect: false },
      ],
    },
    risk: { tier: "question", reason: "question for you" },
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

type AgentSeed = [id: string, type: string, description: string, state: State, tool?: LastTool, done?: { agoS: number; tookS: number; tools: number; result: string }];

// ingest-01 has a child holding the shell card; research-02 overflows, has one lingering and three folded.
const KIDS: Record<number, AgentSeed[]> = {
  0: [
    ["demo-agent-tests", "general-purpose", "Run the row tests", "waiting", { name: "Bash", target: SHELL_CMD }],
    ["demo-agent-schema", "Explore", "Find the line item schema", "working", { name: "Bash", target: ".venv/Scripts/python - <<'EOF'" }],
  ],
  1: [
    ["demo-agent-a", "Explore", "Find vendor_id callers", "working", { name: "Grep", target: "vendor_id" }],
    ["demo-agent-b", "Explore", "Map the vendor docs", "thinking"],
    ["demo-agent-c", "Explore", "Check the API limits", "working", { name: "WebFetch", target: "vendor docs/api" }],
    ["demo-agent-d", "Plan", "Draft the rollout", "thinking"],
    ["demo-agent-e", "general-purpose", "Summarise open issues", "working", { name: "Read", target: "docs/vendors.md" }],
    ["demo-agent-f", "Explore", "List the webhooks", "success", undefined, { agoS: 4, tookS: 72, tools: 6, result: "Found 3 webhooks in src/vendor/hooks.ts" }],
    ["demo-agent-g", "Explore", "Read the changelog", "success", undefined, { agoS: 90, tookS: 41, tools: 3, result: "v2 drops the batch endpoint" }],
    ["demo-agent-h", "Plan", "Order the migrations", "success", undefined, { agoS: 120, tookS: 95, tools: 0, result: "Migrate auth first, then billing" }],
    ["demo-agent-i", "general-purpose", "Count failing jobs", "success", undefined, { agoS: 200, tookS: 30, tools: 4, result: "12 jobs failed since Monday" }],
  ],
};

function kidsFor(i: number, now: number): Agent[] | undefined {
  return KIDS[i]?.map(([id, agentType, description, state, lastTool, done], k) => {
    const endedAt = done ? now - done.agoS * 1000 : undefined;
    const startedAt = done ? endedAt! - done.tookS * 1000 : now - (k + 1) * 20_000;
    const task = state === "working" && lastTool ? `${lastTool.name} ${lastTool.target}` : state === "waiting" ? asks(lastTool!.name) : done ? "Done" : "Thinking";
    return { id, agentType, description, state, stateSince: endedAt ?? now - 5_000, task, lastTool, startedAt, endedAt, toolCount: done?.tools ?? k + 1, result: done?.result };
  });
}

type ShellSeed = [id: string, command: string, description: string, state: Shell["state"], ageS: number, extra?: { endedAgoS?: number; port?: number; exitCode?: number; lastLine?: string }];

// A dev server with its port, a long running command, an exit that folds in ~27s, a failed build that flags its parent.
const SHELLS: Record<number, ShellSeed[]> = {
  0: [
    ["demo-sh-dev", "npm run dev -- --port 5173", "Start the dev server", "serving", 240, { port: 5173, lastLine: "  ➜  Local:   http://localhost:5173/" }],
    ["demo-sh-watch", ".venv/Scripts/python -m pipelines.ingest.watch --batch 14 --source invoices/inbox", "Watch the invoice inbox", "running", 75],
  ],
  3: [
    ["demo-sh-render", "python scripts/render_digest.py --week 46", "Render the digest", "exited", 20, { endedAgoS: 3, exitCode: 0 }],
    ["demo-sh-lint", "npx markdownlint digest/", "Lint the digest", "exited", 300, { endedAgoS: 200, exitCode: 0 }],
  ],
  4: [
    ["demo-sh-tsc", "npx tsc --watch --noEmit", "Type-check in watch mode", "quiet", 600],
    ["demo-sh-build", "npm run build", "Build the sync bundle", "failed", 40, { endedAgoS: 12, exitCode: 1, lastLine: "error TS2307: Cannot find module './sync/queue'" }],
  ],
};

function shellsFor(i: number, now: number): Shell[] | undefined {
  return SHELLS[i]?.map(([id, command, description, state, ageS, x = {}]) => {
    const endedAt = x.endedAgoS !== undefined ? now - x.endedAgoS * 1000 : undefined;
    return { id, command, description, state, stateSince: endedAt ?? now - ageS * 500, startedAt: now - ageS * 1000, endedAt, port: x.port, exitCode: x.exitCode, lastLine: x.lastLine };
  });
}

// AV-66: linked tickets and running Peep skills; the skill ejects when the turn ends, as on Stop.
const LINKS: Record<number, Pick<Session, "skill" | "ticket">> = {
  0: { skill: "qt" },
  3: { skill: "shoot", ticket: { id: "AV-34", title: "chip contrast on the light theme" } },
  4: { skill: "ticket", ticket: { id: "AV-66" } },
};

// AV-68 shelves: normal (one of each status), long (17, scrolls) and empty; triage-03 and scrape-06 have no repo.
const NORMAL: RepoTicket[] = [
  { id: "AV-35", title: "strip clash bracket", status: "open" },
  { id: "AV-31", title: "token sheet", status: "blocked", reason: "the palette is not signed off", wake: "the owner rules on AV-63" },
  { id: "AV-29", title: "sound pack", status: "deferred", reason: "waiting on the emote sheet", wake: "AV-31 is done" },
  { id: "AV-34", title: "chip contrast", status: "doing" },
  { id: "AV-36", title: "tray overflow box", status: "done" },
  { id: "AV-37", title: "first-run disk box", status: "open" },
];
const LONG: RepoTicket[] = [
  ...NORMAL,
  ...["sticker slots", "emote pack loader", "custom floppy renderer", "settings drag order", "per-project colour", "nudge snooze", "eject timer", "hover card delay", "disk box grid", "plank slot art", "shelf scroll footer"].map(
    (title, k): RepoTicket => ({ id: `AV-${80 + k}`, title, status: k % 4 === 3 ? "deferred" : "open", ...(k % 4 === 3 ? { reason: "later", wake: "2026-12-01 passes" } : {}) }),
  ),
];
const REPOS: Repo[] = [
  { root: "C:/Users/demo/Projects/accounts-payable", sessions: ["demo-1", "demo-4"], tickets: NORMAL },
  { root: "C:/demo/sync-05", sessions: ["demo-5"], tickets: LONG },
  { root: "C:/demo/research-02", sessions: ["demo-2"], tickets: [] },
];

const skillIn = (i: number, state: State) => (state === "success" || state === "idle" ? undefined : LINKS[i]?.skill);

function arm(p: Omit<PendingApproval, "receivedAt" | "releaseAt">, now: number): PendingApproval {
  return { ...p, receivedAt: now, releaseAt: now + HOLD_MS };
}

const demoId = (i: number) => `demo-${i + 1}`;
const identities = new Map(AGENTS.map((a, i) => [demoId(i), a.identity]));

export function demoIdentity(s: Session): Identity {
  return identities.get(s.id) ?? { color: "yellow", accessory: "none" };
}

function taskFor(i: number, state: State): string {
  const tool = AGENTS[i].tool;
  if (state === "working") return `${tool.name} ${tool.target}`;
  return state === "waiting" ? asks(tool.name) : TASKS[state];
}

export function demoSnapshot(now: number): Snapshot {
  return {
    integration: "missing",
    rateLimits: { fiveHourPct: 42, sevenDayPct: 18, at: now },
    repos: REPOS,
    sessions: AGENTS.map((a, i) => withPending({
      id: demoId(i),
      pid: 1000 + i,
      name: a.name,
      cwd: i === 0 ? "C:/Users/demo/Projects/accounts-payable/pipelines/ingest-01" : `C:/demo/${a.name}`,
      kind: "interactive",
      entrypoint: i === 2 ? "claude-vscode" : "cli",
      model: i === 1 ? undefined : "Opus 5.5",
      lastTool: a.tool,
      state: a.state,
      stateSince: now - i * 60_000,
      task: taskFor(i, a.state),
      subagents: KIDS[i]?.filter((k) => !k[5]).length ?? 0,
      agents: kidsFor(i, now),
      shells: shellsFor(i, now),
      // Low, amber (70+) and red (90+) context for the segmented bars.
      contextPct: [74, 35, 93, 12, 47, undefined][i],
      alive: true,
      ticket: LINKS[i]?.ticket,
      skill: skillIn(i, a.state),
    }, i, now)),
  };
}

/** `?demo&digest`: a sample away card over the demo sessions. */
export function demoDigest(snap: Snapshot, now: number): Digest {
  const [a, b, c, d] = snap.sessions;
  const base = { alive: true, pending: false, files: [], fileCount: 0, commands: 0, released: 0, partial: false };
  return {
    since: now - 42 * 60_000,
    until: now,
    entries: [
      { ...base, sessionId: a.id, name: a.name, group: "blocked", state: "waiting", pending: true, tookMs: 18 * 60_000, commands: 3, released: 1 },
      { ...base, sessionId: b.id, name: b.name, group: "failed", state: "error", tookMs: 6 * 60_000, commands: 7, lastLine: "API error: 529 Overloaded" },
      {
        ...base,
        sessionId: c.id,
        name: c.name,
        group: "finished",
        state: "success",
        tookMs: 24 * 60_000,
        files: ["src/rows.ts", "src/styles.css", "src/digest.ts"],
        fileCount: 3,
        commands: 12,
        lastLine: "All 175 tests pass and the build is clean.",
        summary: "Restyled the session rows and the digest, then confirmed tests and build pass.",
      },
      { ...base, sessionId: d.id, name: d.name, group: "running", state: "working", tookMs: 31 * 60_000, files: ["src-tauri/src/away.rs"], fileCount: 1, commands: 4 },
    ],
  };
}

function withPending(s: Session, i: number, now: number): Session {
  const p = PENDING[i];
  return p ? { ...s, state: "waiting", task: asks(p.tool), pending: [arm(p, now)] } : s;
}

/** Drops a decided demo request; the session moves on as the backend would. */
export function demoDecide(snap: Snapshot, requestId: string, decision: "allow" | "deny", now: number): Snapshot {
  const sessions = snap.sessions.map((s, i) => {
    const decided = s.pending?.find((p) => p.id === requestId);
    if (!decided) return s;
    const pending = s.pending!.filter((p) => p.id !== requestId);
    const state: State = decision === "allow" ? "working" : "thinking";
    const agents = s.agents?.map((a) =>
      a.id === decided.agentId ? { ...a, state, stateSince: now, task: state === "working" ? `${decided.tool} ${SHELL_CMD}` : "Thinking" } : a,
    );
    if (pending.length) return { ...s, pending, agents };
    return { ...s, pending: undefined, agents, state, stateSince: now, task: taskFor(i, state) };
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
  sessions[i] = { ...sessions[i], state: next, stateSince: now, task: taskFor(i, next), skill: skillIn(i, next) };
  const rateLimits = snap.rateLimits && { ...snap.rateLimits, at: now };
  return { ...snap, sessions, rateLimits };
}
