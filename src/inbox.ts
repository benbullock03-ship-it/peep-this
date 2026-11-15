import { byOldest } from "./approval";
import type { PendingApproval, Session } from "./model";

/** One pending request in keyboard-inbox order (AV-24); subagent requests carry `agentId`. */
export type InboxItem = Pick<PendingApproval, "id" | "receivedAt" | "agentId"> & { sessionId: string };

const isQuestion = (p: PendingApproval) => Number(p.risk.tier === "question");

/** Every live request across sessions and their subagents, oldest first; questions queue after real approvals. */
export function inboxOrder(sessions: readonly Session[]): InboxItem[] {
  const all: (PendingApproval & { sessionId: string })[] = [];
  for (const s of sessions) if (s.alive) for (const p of s.pending ?? []) all.push({ ...p, sessionId: s.id });
  return all.sort((a, b) => isQuestion(a) - isQuestion(b) || byOldest(a, b)).map(({ id, receivedAt, agentId, sessionId }) => ({ id, receivedAt, agentId, sessionId }));
}

/** Tab / Shift+Tab, wrapping. An unknown current starts from the matching end. */
export function step(order: readonly InboxItem[], current: string | null, dir: 1 | -1): string | null {
  if (order.length === 0) return null;
  const i = order.findIndex((x) => x.id === current);
  if (i < 0) return (dir > 0 ? order[0] : order[order.length - 1]).id;
  return order[(i + dir + order.length) % order.length].id;
}

/**
 * Which card holds focus once `next` replaces `prev`: the current one while it is still pending,
 * otherwise the first request after it in the old order that is still pending, otherwise the oldest.
 * Null means the inbox is empty and focus goes back to where the user came from.
 */
export function successor(prev: readonly InboxItem[], next: readonly InboxItem[], current: string | null): string | null {
  if (next.length === 0) return null;
  const live = new Set(next.map((x) => x.id));
  if (current && live.has(current)) return current;
  const i = prev.findIndex((x) => x.id === current);
  if (i >= 0) for (const x of prev.slice(i + 1)) if (live.has(x.id)) return x.id;
  return next[0].id;
}

export function badgeText(n: number): string {
  return n > 9 ? "9+" : String(n);
}

const NAMED: Record<string, string> = {
  Space: "Space",
  Home: "Home",
  End: "End",
  Insert: "Insert",
  Delete: "Delete",
  PageUp: "PageUp",
  PageDown: "PageDown",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
};

type KeyLike = Pick<KeyboardEvent, "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">;

/** A pressed combination in the backend's canonical form, or null while only modifiers are down. */
export function comboFromKey(e: KeyLike): string | null {
  const c = e.code;
  const key = /^Key[A-Z]$/.test(c)
    ? c.slice(3)
    : /^Digit\d$/.test(c)
      ? c.slice(5)
      : /^F([1-9]|1\d|2[0-4])$/.test(c)
        ? c
        : NAMED[c];
  if (!key) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Win"].filter(Boolean);
  return [...mods, key].join("+");
}
