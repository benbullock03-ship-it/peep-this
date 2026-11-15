import type { Mood, Session, State } from "./model";
import { isAttention } from "./stateMeta";

export type Play = "loop" | "once" | "twice";

/** design/skill-pack/emotes.json: sheet row and how it plays. `once` holds frame 0 after; `twice` hides after. */
export const EMOTES = {
  sweat: { row: 1, play: "loop" },
  yawn: { row: 2, play: "once" },
  sparkle: { row: 3, play: "twice" },
  wave: { row: 4, play: "twice" },
  worry: { row: 5, play: "loop" },
  zzz: { row: 6, play: "loop" },
} as const satisfies Record<string, { row: number; play: Play }>;

export type Emote = keyof typeof EMOTES;

export const MOOD_EMOTE: Record<Mood, Emote> = {
  cheer: "sparkle",
  greet: "wave",
  worry: "worry",
  yawn: "yawn",
  sweat: "sweat",
  nap: "zzz",
};

/** The bubble's emote; none while an attention state holds the bubble or once the session has closed. */
export function emoteFor(s: Pick<Session, "mood" | "alive">, state: State): Emote | null {
  if (!s.alive || !s.mood || isAttention(state)) return null;
  return MOOD_EMOTE[s.mood] ?? null;
}

export const LAST_NUDGE_PCT = 95;

export type Nudge = "first" | "last";

/** FLOPPY-SPEC §09: the yawn bubble suggests Pack Up; "Not yet" snoozes it until the last nudge at 95%. */
export function nudgeFor(s: Pick<Session, "mood" | "alive" | "contextPct">, state: State, snoozed: boolean): Nudge | null {
  if (emoteFor(s, state) !== "yawn") return null;
  if ((s.contextPct ?? 0) >= LAST_NUDGE_PCT) return "last";
  return snoozed ? null : "first";
}

export function nudgeText(n: Nudge, pct: number | undefined): string {
  return n === "last" ? "Nearly out of room. Pack up now and nothing gets lost." : `Getting sleepy. I'm ${Math.round(pct ?? 0)}% full.`;
}

/** Mirrors `mood::valid_nudge`. */
export const NUDGE_RANGE = { min: 50, max: 95, step: 5, initial: 80 } as const;
