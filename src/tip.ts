import type { Session, State, Tip } from "./model";

export const TIP_TEXT: Record<Tip, string> = {
  crowded: "Getting crowded; answers get fuzzier from here.",
  misses: "Two misses? /clear with what you learned beats a third try.",
  cold: "Cache went cold; the next reply re-reads everything.",
  helper: "That helper reads in its own window and sends back a summary.",
  compacted: "Compacted: CLAUDE.md and recent files kept, details lost. That is why Pack Up exists.",
};

/** Never while busy or in an attention state; a nudge takes the slot first. `dismissed` is the tip this row already waved off. */
export function tipFor(s: Pick<Session, "tip" | "alive">, state: State, nudged: boolean, dismissed?: Tip): Tip | null {
  if (!s.tip || !s.alive || nudged || s.tip === dismissed) return null;
  return state === "idle" || state === "success" ? s.tip : null;
}
