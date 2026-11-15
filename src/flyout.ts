/** AV-42: which waiting sessions fly out of the strip, and where their rows sit (spec §12.7). */
import type { Session } from "./model";
import { sortSessions } from "./sort";
import { STRIP_MAX } from "./strip";

/** Mirrors `Side` in src-tauri/src/flyout.rs: where the flyouts sit, seen from the strip. */
export type Side = "left" | "right" | "up" | "down";

/** `slot` is the strip cell the session sits in, or `max` for the `+N` tile. */
export type FlyWant = { id: string; slot: number };

/** A click dismisses a flyout until its session next enters `waiting` (a new `stateSince`). */
export type Dismissed = Map<string, number>;

export function flyWants(sessions: readonly Session[], show: boolean, dismissed: ReadonlyMap<string, number>, max = STRIP_MAX): FlyWant[] {
  if (!show) return [];
  return sortSessions(sessions).flatMap((s, i) =>
    s.alive && s.state === "waiting" && dismissed.get(s.id) !== s.stateSince ? [{ id: s.id, slot: Math.min(i, max) }] : [],
  );
}

/** Forgets dismissals whose waiting spell is over. */
export function pruneDismissed(dismissed: Dismissed, sessions: readonly Session[]) {
  for (const [id, since] of dismissed) {
    const s = sessions.find((x) => x.id === id);
    if (!s || s.state !== "waiting" || s.stateSince !== since) dismissed.delete(id);
  }
}

/** Matches the `.ps` hard shadow. */
export const SHADOW = 6;
/** Between the strip and its flyouts; flyout.rs ZONE_W counts it. */
export const GAP = 10;
export const STACK_GAP = 6;
/** The tail stays this far from a card's corners. */
const TAIL_INSET = 12;

/** `w`/`h` include the shadow; `at` is the peep's centre along the strip, in zone px. */
export type FlyBox = { id: string; w: number; h: number; at: number };
/** `tail` is where the pointer sits along the edge facing the strip, or null when another flyout is in between. */
export type Placed = { id: string; x: number; y: number; tail: number | null };

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

/**
 * Lays the flyouts out in a zone beside the strip, in strip order. Beside a column each sits level
 * with its peep, pushed down past the one before; off a row they stack away from the strip, each
 * under its own peep.
 */
export function layOut(side: Side, boxes: readonly FlyBox[], zone: { w: number; h: number }): Placed[] {
  const tailAt = (at: number, start: number, len: number) => clamp(at - start, TAIL_INSET, len - SHADOW - TAIL_INSET);
  if (side === "left" || side === "right") {
    const tops: number[] = [];
    let floor = 0;
    for (const b of boxes) {
      const top = Math.max(b.at - (b.h - SHADOW) / 2, floor);
      tops.push(top);
      floor = top + b.h + STACK_GAP;
    }
    const over = boxes.length ? tops[tops.length - 1] + boxes[boxes.length - 1].h - zone.h : 0;
    const lift = over > 0 ? Math.min(over, tops[0] ?? 0) : 0;
    return boxes.map((b, i) => {
      const y = Math.max(0, tops[i] - lift);
      return { id: b.id, x: side === "left" ? zone.w - GAP - b.w : GAP, y, tail: tailAt(b.at, y, b.h) };
    });
  }
  let edge = side === "down" ? GAP : zone.h - GAP;
  return boxes.map((b, i) => {
    const x = clamp(b.at - (b.w - SHADOW) / 2, 0, Math.max(0, zone.w - b.w));
    const y = side === "down" ? edge : edge - b.h;
    edge = side === "down" ? y + b.h + STACK_GAP : y - STACK_GAP;
    return { id: b.id, x, y, tail: i === 0 ? tailAt(b.at, x, b.w) : null };
  });
}
