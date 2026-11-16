import type { Identity } from "./identity";
import type { Agent, ClosedSession, Digest, LastTool, Mood, PendingApproval, Repo, RepoTicket, Session, Shell, Snapshot, State, Tip } from "./model";

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
  {
    root: "C:/Users/demo/Projects/accounts-payable",
    sessions: ["demo-1", "demo-4"],
    tickets: NORMAL,
    northStar: "A calm desk pet that teaches new Claude Code users a rhythm",
    arc: { closed: 41, open: 4, parked: 2, last: { id: "AV-70", title: "cheer when a ticket closes" } },
  },
  // AV-83: no north star (sync-05), and nothing closed yet (research-02). AV-108: research-02 is a fresh repo, so its Extras are locked.
  { root: "C:/demo/sync-05", sessions: ["demo-5"], tickets: LONG, arc: { closed: 6, open: 13, parked: 4, last: { id: "AV-06", title: "" } } },
  {
    root: "C:/demo/research-02", sessions: ["demo-2"], tickets: [], northStar: "Ward maps anyone can read", arc: { closed: 0, open: 0, parked: 0 },
    unlock: { ticketRuns: 1, closedUi: false, test: true, url: false },
  },
];

// AV-70: one row per mood, both nudges, plus sync-05 (error) holding a worry the bubble must hide. They sit out demoStep.
// AV-71: shell-14 and shell-15 clash on one file; equal stateSince keeps them adjacent, so the bracket shows.
const MOODY: { name: string; identity: Identity; state: State; mood: Mood; task: string; contextPct?: number; agoS?: number; ticket?: Session["ticket"]; clash?: string }[] = [
  { name: "docs-07", identity: { color: "blue", accessory: "glasses" }, state: "working", mood: "sweat", task: "Write docs/api.md", contextPct: 76 },
  { name: "billing-08", identity: { color: "pink", accessory: "beanie" }, state: "thinking", mood: "yawn", task: "Checking the invoice totals", contextPct: 82 },
  { name: "ledger-09", identity: { color: "lavender", accessory: "wizard" }, state: "working", mood: "yawn", task: "Edit src/ledger.ts", contextPct: 96 },
  { name: "api-10", identity: { color: "green", accessory: "party" }, state: "idle", mood: "cheer", task: "Idle", ticket: { id: "AV-12", title: "rate limit headers" } },
  { name: "infra-11", identity: { color: "yellow", accessory: "crown" }, state: "thinking", mood: "greet", task: "Reading the deploy script" },
  { name: "tests-12", identity: { color: "blue", accessory: "headphones" }, state: "thinking", mood: "worry", task: "Reading the failing test" },
  { name: "tweets-16", identity: { color: "pink", accessory: "bow" }, state: "working", mood: "tweet", task: "Edit .peep/tweets.md" },
  { name: "notes-13", identity: { color: "green", accessory: "flowers" }, state: "idle", mood: "nap", task: "Idle", agoS: 900 },
  { name: "shell-14", identity: { color: "yellow", accessory: "beanie" }, state: "working", mood: "startled", task: "Edit src/shell.css", agoS: 20, clash: "C:/demo/peep-this/src/shell.css" },
  { name: "shell-15", identity: { color: "lavender", accessory: "glasses" }, state: "working", mood: "startled", task: "Edit src/shell.css", agoS: 20, clash: "C:/demo/peep-this/src/shell.css" },
];
const MOOD_FROM = AGENTS.length;
const ATTENTION_MOOD: Record<number, Mood> = { 4: "worry" };

function moodySessions(now: number): Session[] {
  return MOODY.map((m, k) => ({
    id: demoId(MOOD_FROM + k),
    pid: 1000 + MOOD_FROM + k,
    name: m.name,
    cwd: `C:/demo/${m.name}`,
    kind: "interactive",
    entrypoint: "cli",
    model: "Opus 5.5",
    state: m.state,
    stateSince: now - (m.agoS ?? 30) * 1000,
    task: m.task,
    subagents: 0,
    contextPct: m.contextPct ?? 20 + k * 3,
    mood: m.mood,
    clash: m.clash,
    ticket: m.ticket,
    alive: true,
  }));
}

const skillIn = (i: number, state: State) => (state === "success" || state === "idle" ? undefined : LINKS[i]?.skill);

function arm(p: Omit<PendingApproval, "receivedAt" | "releaseAt">, now: number): PendingApproval {
  return { ...p, receivedAt: now, releaseAt: now + HOLD_MS };
}

const demoId = (i: number) => `demo-${i + 1}`;
const identities = new Map([...AGENTS, ...MOODY].map((a, i) => [demoId(i), a.identity]));

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
      title: [undefined, "Survey OCR papers", "Fix label colours", undefined, "Nightly sync retries", undefined][i],
      branch: ["main", "research/ocr", "fix/label-colours-for-dark-theme", undefined, "main", undefined][i],
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
      mood: ATTENTION_MOOD[i],
    }, i, now)).concat(moodySessions(now)),
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

/** `?demo&end=fail` stubs `end_session` refusing with AV-96's mismatch error; otherwise the session's process dies. */
export function demoEnd(snap: Snapshot, sessionId: string, mode: string | null): Snapshot {
  if (mode === "fail") throw "the PID now belongs to another process";
  return { ...snap, sessions: snap.sessions.map((s) => (s.id === sessionId ? { ...s, alive: false, pending: undefined } : s)) };
}

type ClosedSeed =[name: string, state: State, agoM: number, bubble: string, ticket: string | null, flags: string];

// design/skill-pack-2/ref/ClosedShelf.dc.html rows; flags: u uncommitted, d ticket left doing, p no Pack Up.
const CLOSED: ClosedSeed[] = [
  ["polling-91", "waiting", 14, "Allow rm -rf dist?", "P-12", "udp"],
  ["peep-this-6c", "success", 60, "AV-70: mood emotes · packed up", "AV-70", ""],
  ["siambr-01", "error", 120, "API error 529: overloaded", "W-71", "d"],
  ["datamapuk-3d", "success", 180, "S-308: sitemap index · packed up", "S-308", ""],
  ["peep-this-5f", "idle", 300, "All done!", "AV-68", ""],
  ["polling-90", "success", 420, "P-11: poll card copy · packed up", "P-11", ""],
];
const CLOSED_LONG: ClosedSeed = [
  "accounts-payable-ingest-pipeline-normaliser-07",
  "stuck",
  25,
  "Reading pipelines/ingest/invoices/batch-14/normalise_line_items.py and the vendor schema it maps onto",
  "AV-104",
  "udp",
];

function closedFrom([name, state, agoM, bubble, ticket, flags]: ClosedSeed, i: number, now: number): ClosedSession {
  return {
    sessionId: `3f9a2c1e-0000-4000-8000-${String(i).padStart(12, "0")}`,
    name,
    cwd: `C:/demo/${name.replace(/-[0-9a-f]+$/, "")}`,
    bubble,
    state,
    ticket: ticket ? { id: ticket } : undefined,
    endedAt: now - agoM * 60_000,
    flags: { uncommitted: flags.includes("u"), ticketDoing: flags.includes("d"), noPackup: flags.includes("p"), attention: ["waiting", "error", "stuck"].includes(state) },
  };
}

/** `?demo&closed=none|one|many|long`: empty, one row, the design's six, or twenty with an overlong first row. Default many. */
export function demoClosed(now: number, mode: string | null): ClosedSession[] {
  if (mode === "none") return [];
  if (mode === "one") return [closedFrom(CLOSED[0], 0, now)];
  if (mode !== "long") return CLOSED.map((c, i) => closedFrom(c, i, now));
  const seeds = [CLOSED_LONG, ...Array.from({ length: 19 }, (_, i): ClosedSeed => {
    const [name, state, , bubble, ticket, flags] = CLOSED[i % CLOSED.length];
    return [`${name}-${i}`, state, 30 + i * 22, bubble, ticket, i % 4 === 2 ? "p" : flags];
  })];
  return seeds.map((c, i) => closedFrom(c, i, now));
}

const BRIEF: readonly (Mood | undefined)[] = ["startled", "cheer", "greet"];

/** `?demo&limits=82,30`: those limits, resetting in 1h 12m and 3 days, with the sweat `mood.rs` would give from 80%. */
export function demoLimits(snap: Snapshot, fiveHourPct: number, sevenDayPct: number, now: number): Snapshot {
  const rateLimits = { fiveHourPct, sevenDayPct, fiveHourResetsAt: now + 72 * 60_000, sevenDayResetsAt: now + 3 * 86_400_000, at: now };
  if (Math.max(fiveHourPct, sevenDayPct) < 80) return { ...snap, rateLimits };
  const sessions = snap.sessions.map((s) => (s.alive && !BRIEF.includes(s.mood) ? { ...s, mood: "sweat" as const } : s));
  return { ...snap, sessions, rateLimits };
}

/** `?demo&tip=<name>`: every session holds the tip; only idle and done rows show it. */
export function demoTips(snap: Snapshot, tip: Tip): Snapshot {
  return { ...snap, sessions: snap.sessions.map((s) => ({ ...s, tip })) };
}

// Mirrors the design's simulation: one agent changes state per step.
export function demoStep(snap: Snapshot, now: number, rand: () => number = Math.random): Snapshot {
  // Held requests re-arm instead of releasing, so the cards stay up for a look.
  const sessions = snap.sessions.map((s) =>
    s.pending?.length ? { ...s, pending: s.pending.map((p) => (p.releaseAt <= now ? { ...p, receivedAt: now, releaseAt: now + HOLD_MS } : p)) } : s,
  );
  const free = sessions.flatMap((s, k) => (s.pending?.length || k >= MOOD_FROM ? [] : [k]));
  if (!free.length) return { ...snap, sessions, rateLimits: snap.rateLimits && { ...snap.rateLimits, at: now } };
  const i = free[Math.floor(rand() * free.length)];
  let next: State;
  do next = FLOW[Math.floor(rand() * FLOW.length)];
  while (next === sessions[i].state);
  sessions[i] = { ...sessions[i], state: next, stateSince: now, task: taskFor(i, next), skill: skillIn(i, next) };
  const rateLimits = snap.rateLimits && { ...snap.rateLimits, at: now };
  return { ...snap, sessions, rateLimits };
}
