import type { Mood, Session, State } from "./model";
import { isAttention } from "./stateMeta";

export type Play = "loop" | "once" | "twice" | "twice-hold";

/** design/skill-pack/emotes.json: sheet row and how it plays. `once` holds frame 0 after; `twice` hides after; `twice-hold` holds frame 1. */
export const EMOTES = {
  startled: { row: 0, play: "twice-hold" },
  sweat: { row: 1, play: "loop" },
  yawn: { row: 2, play: "once" },
  sparkle: { row: 3, play: "twice" },
  wave: { row: 4, play: "twice" },
  worry: { row: 5, play: "loop" },
  zzz: { row: 6, play: "loop" },
} as const satisfies Record<string, { row: number; play: Play }>;

export type Emote = keyof typeof EMOTES;

export const MOOD_EMOTE: Record<Mood, Emote> = {
  startled: "startled",
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

/** A clash as one bubble shows it: the bolt tag letter, the file's name and how many sessions share it. */
export type ClashTag = { tag: string; file: string; count: number };

const baseName = (path: string) => path.split(/[\\/]/).filter(Boolean).pop() ?? path;

/** ADR 0002: bolt tags A, B… per clashed file in list order, for the sessions whose bubble shows the startled emote. */
export function clashTags(sorted: readonly Session[]): Map<string, ClashTag> {
  const out = new Map<string, ClashTag>();
  const files: string[] = [];
  for (const s of sorted) {
    if (!s.clash || emoteFor(s, s.state) !== "startled") continue;
    if (!files.includes(s.clash)) files.push(s.clash);
    const count = sorted.filter((o) => o.alive && o.clash === s.clash).length;
    out.set(s.id, { tag: String.fromCharCode(65 + (files.indexOf(s.clash) % 26)), file: baseName(s.clash), count });
  }
  return out;
}

export const clashText = (c: ClashTag) => (c.count > 2 ? `All ${c.count} in ${c.file}` : `Both in ${c.file}`);

/** `true` at i when row i and row i+1 share a bolt tag: the dotted bracket joins only rows that already sit together. */
export function clashLinks(sorted: readonly Session[], tags: ReadonlyMap<string, ClashTag>): boolean[] {
  return sorted.map((s, i) => {
    const a = tags.get(s.id)?.tag;
    return !!a && i + 1 < sorted.length && tags.get(sorted[i + 1].id)?.tag === a;
  });
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
