import { identityFor, type Identity } from "./identity";
import { STATES, type Session, type State } from "./model";
import { sayFor } from "./say";
import { failNote } from "./shells";
import { isAttention, STATE_LABEL } from "./stateMeta";

export type RowView = {
  id: string;
  name: string;
  say: string;
  state: State;
  label: string;
  attention: boolean;
  color: string;
  accessory: string;
  contextPct: number | null;
  alive: boolean;
  /** Non-empty when a background shell failed: the red dot's hover text. */
  shellNote: string;
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
    console.error("peepthis: focus_session failed", err);
  }
  if (!ok) flash();
}

export function toRowView(s: Session, identityOf: IdentityOf = (x) => identityFor(x.cwd), now = Date.now()): RowView {
  const state: State = STATES.includes(s.state) ? s.state : "idle";
  const { color, accessory } = identityOf(s);
  return {
    id: s.id,
    name: s.name || s.id.slice(0, 8),
    say: sayFor({ ...s, state }, now),
    state,
    label: STATE_LABEL[state],
    attention: isAttention(state),
    color,
    accessory,
    contextPct: contextFill(s.contextPct),
    alive: s.alive,
    shellNote: failNote(s.shells),
  };
}
