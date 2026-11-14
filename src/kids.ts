import { formatDuration } from "./details";
import { ACCESSORIES, fnv1a, type Accessory } from "./identity";
import type { Agent, PendingApproval, State } from "./model";
import { kidSayFor } from "./say";
import { isAttention, STATE_LABEL } from "./stateMeta";

/** A finished child stays as its own row this long, then folds into "done N". */
export const KID_LINGER_MS = 30_000;
/** Running children shown before the "+N more" row. */
export const KID_MAX_RUNNING = 4;

// "none" is the empty-state house peep's look, so a child always wears something.
const KID_ACCESSORIES = ACCESSORIES.filter((a) => a !== "none");

/** Same agent type, same accessory, in every project; colour comes from the parent. */
export function kidAccessory(agentType: string): Accessory {
  const key = agentType.trim().toLowerCase() || "agent";
  return KID_ACCESSORIES[fnv1a(key) % KID_ACCESSORIES.length];
}

export const isFinished = (a: Agent) => a.endedAt !== undefined;
export const isFolded = (a: Agent, now: number) => a.endedAt !== undefined && now - a.endedAt >= KID_LINGER_MS;

export type KidPlan = {
  /** Rows to draw, in order. */
  shown: Agent[];
  /** Running children hidden behind "+N more" (0 when none or expanded). */
  more: number;
  /** Running children past the cap, whether or not expanded; decides if the toggle shows. */
  overflow: number;
  /** Finished children past their linger, behind the "done N" counter. */
  done: number;
};

export type PlanOpts = { allRunning?: boolean; showDone?: boolean; pending?: readonly PendingApproval[] };

export function planKids(agents: readonly Agent[] | undefined, now: number, opts: PlanOpts = {}): KidPlan {
  const asking = new Set((opts.pending ?? []).map((p) => p.agentId).filter(Boolean));
  const rank = (a: Agent) => (asking.has(a.id) ? 0 : isAttention(a.state) ? 1 : 2);
  const running = (agents ?? [])
    .filter((a) => !isFinished(a))
    .sort((a, b) => rank(a) - rank(b) || a.startedAt - b.startedAt || a.id.localeCompare(b.id));
  const newestFirst = (a: Agent, b: Agent) => b.endedAt! - a.endedAt! || a.id.localeCompare(b.id);
  const finished = (agents ?? []).filter(isFinished);
  const lingering = finished.filter((a) => !isFolded(a, now)).sort(newestFirst);
  const folded = finished.filter((a) => isFolded(a, now)).sort(newestFirst);
  const overflow = Math.max(0, running.length - KID_MAX_RUNNING);
  const visibleRunning = opts.allRunning ? running : running.slice(0, KID_MAX_RUNNING);
  return {
    shown: [...visibleRunning, ...lingering, ...(opts.showDone ? folded : [])],
    more: opts.allRunning ? 0 : overflow,
    overflow,
    done: folded.length,
  };
}

/** When the next lingering child folds, so the list can re-render without a new snapshot. */
export function nextFoldAt(agents: readonly Agent[] | undefined, now: number): number | null {
  let next: number | null = null;
  for (const a of agents ?? []) {
    if (a.endedAt === undefined || isFolded(a, now)) continue;
    const at = a.endedAt + KID_LINGER_MS;
    if (next === null || at < next) next = at;
  }
  return next;
}

export type KidView = {
  id: string;
  name: string;
  description: string;
  say: string;
  state: State;
  label: string;
  attention: boolean;
  accessory: Accessory;
  finished: boolean;
  /** Finished only: "1m 04s · 3 tools". */
  stats: string;
};

export function kidView(a: Agent, pending: readonly PendingApproval[] | undefined, now: number): KidView {
  const mine = (pending ?? []).filter((p) => p.agentId === a.id);
  const finished = isFinished(a);
  const state: State = mine.length && !finished ? "waiting" : a.state;
  const tools = `${a.toolCount} ${a.toolCount === 1 ? "tool" : "tools"}`;
  return {
    id: a.id,
    name: a.agentType.trim() || "agent",
    description: a.description.trim(),
    say: kidSayFor({ ...a, state }, mine, now),
    state,
    label: finished && state === "idle" ? "Stopped" : STATE_LABEL[state] ?? state,
    attention: !finished && isAttention(state),
    accessory: kidAccessory(a.agentType),
    finished,
    stats: finished ? `${formatDuration(a.endedAt! - a.startedAt)} · ${tools}` : "",
  };
}
