import { identityFor, type Identity } from "./identity";
import { STATES, type Session, type State } from "./model";
import { STATE_META } from "./stateMeta";

export type RowView = {
  id: string;
  name: string;
  task: string;
  state: State;
  label: string;
  led: string;
  color: string;
  accessory: string;
  subagents: number;
  contextPct: number | null;
  alive: boolean;
};

export type IdentityOf = (s: Session) => Identity;

export function contextFill(pct: number | null | undefined): number | null {
  if (pct === undefined || pct === null || !Number.isFinite(pct)) return null;
  return Math.max(0, Math.min(100, pct));
}

export type OnFocus = (sessionId: string) => Promise<boolean>;

/** Calls `onFocus`; `flash` runs when no window came forward (false or a rejected call). */
export async function requestFocus(onFocus: OnFocus, id: string, flash: () => void): Promise<void> {
  let ok = false;
  try {
    ok = await onFocus(id);
  } catch (err) {
    console.error("agentview: focus_session failed", err);
  }
  if (!ok) flash();
}

export function toRowView(s: Session, identityOf: IdentityOf = (x) => identityFor(x.cwd)): RowView {
  const state: State = STATES.includes(s.state) ? s.state : "idle";
  const meta = STATE_META[state];
  const { color, accessory } = identityOf(s);
  return {
    id: s.id,
    name: s.name || s.id.slice(0, 8),
    task: s.task || meta.label,
    state,
    label: meta.label,
    led: meta.led,
    color,
    accessory,
    subagents: Math.max(0, Math.floor(s.subagents || 0)),
    contextPct: contextFill(s.contextPct),
    alive: s.alive,
  };
}
