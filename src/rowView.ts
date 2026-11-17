import { BLANK_COL, bubbleFor, bubbleLine, FLOPPIES, signFor, type Bubble } from "./bubble";
import { clashText, emoteFor, nudgeFor, nudgeText, type ClashTag, type Emote, type Nudge } from "./emote";
import { identityFor, type Identity } from "./identity";
import type { LimitWarning } from "./limits";
import { STATES, type Session, type State, type Tip } from "./model";
import { sayFor } from "./say";
import { failNote } from "./shells";
import { isAttention, STATE_LABEL } from "./stateMeta";
import { TIP_TEXT, tipFor } from "./tip";

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
  /** Non-empty when the session file did not parse: the `?` chip's hover text. */
  unparsedNote: string;
  /** Non-empty when a background shell failed: the red dot's hover text. */
  shellNote: string;
  emote: Emote | null;
  nudge: Nudge | null;
  tip: Tip | null;
};

export type IdentityOf = (s: Session) => Identity;

export const UNPARSED_NOTE = "Session file format not understood; the row may lag behind the session";

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

/**
 * `snoozed`: "Not yet" was pressed on this session's first nudge. `clash`: this row's entry from `clashTags`. `dismissed`: the tip "Got it" waved off.
 * `setup`: the session's repo has `.peep/setup.md`; undefined when the repo is unknown.
 */
export function toRowView(
  s: Session,
  identityOf: IdentityOf = (x) => identityFor(x.cwd),
  now = Date.now(),
  snoozed = false,
  clash?: ClashTag,
  limit?: LimitWarning | null,
  dismissed?: Tip,
  setup?: boolean,
): RowView {
  const state: State = STATES.includes(s.state) ? s.state : "idle";
  const { color, accessory } = identityOf(s);
  const attention = isAttention(state);
  const nudge = nudgeFor(s, state, snoozed);
  const tip = tipFor(s, state, !!nudge, dismissed);
  const emote = emoteFor(s, state);
  const sign = nudge || tip || emote ? null : signFor(s, state, setup);
  const bubble: Bubble = nudge
    ? { text: nudgeText(nudge, s.contextPct) }
    : tip
      ? { text: TIP_TEXT[tip] }
      : clash && emote === "startled"
      ? { text: clashText(clash), bolt: clash.tag }
        : sign ?? bubbleFor(s, sayFor({ ...s, state }, now), attention, emote === "sweat" ? limit : null);
  return {
    id: s.id,
    name: s.title || s.name || s.id.slice(0, 8),
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
    unparsedNote: s.unparsed ? UNPARSED_NOTE : "",
    shellNote: failNote(s.shells),
    emote,
    nudge,
    tip,
  };
}
