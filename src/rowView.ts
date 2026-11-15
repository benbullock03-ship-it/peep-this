import { BLANK_COL, bubbleFor, bubbleLine, FLOPPIES, type Bubble } from "./bubble";
import { emoteFor, nudgeFor, nudgeText, type Emote, type Nudge } from "./emote";
import { identityFor, type Identity } from "./identity";
import { STATES, type Session, type State } from "./model";
import { sayFor } from "./say";
import { failNote } from "./shells";
import { isAttention, STATE_LABEL } from "./stateMeta";

export type RowView = {
  id: string;
  name: string;
  say: string;
  bubble: Bubble;
  /** Sprite-sheet column of the inserted floppy; null when no Peep skill runs. */
  floppy: number | null;
  state: State;
  label: string;
  attention: boolean;
  color: string;
  accessory: string;
  contextPct: number | null;
  alive: boolean;
  /** Non-empty when a background shell failed: the red dot's hover text. */
  shellNote: string;
  emote: Emote | null;
  nudge: Nudge | null;
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

/** `snoozed`: "Not yet" was pressed on this session's first nudge. */
export function toRowView(s: Session, identityOf: IdentityOf = (x) => identityFor(x.cwd), now = Date.now(), snoozed = false): RowView {
  const state: State = STATES.includes(s.state) ? s.state : "idle";
  const { color, accessory } = identityOf(s);
  const attention = isAttention(state);
  const nudge = nudgeFor(s, state, snoozed);
  const bubble: Bubble = nudge ? { text: nudgeText(nudge, s.contextPct) } : bubbleFor(s, sayFor({ ...s, state }, now), attention);
  return {
    id: s.id,
    name: s.name || s.id.slice(0, 8),
    say: bubbleLine(bubble),
    bubble,
    floppy: s.alive && s.skill ? (FLOPPIES[s.skill]?.col ?? BLANK_COL) : null,
    state,
    label: STATE_LABEL[state],
    attention,
    color,
    accessory,
    contextPct: contextFill(s.contextPct),
    alive: s.alive,
    shellNote: failNote(s.shells),
    emote: emoteFor(s, state),
    nudge,
  };
}
