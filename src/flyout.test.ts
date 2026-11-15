import { describe, expect, it } from "vitest";
import { flyWants, GAP, layOut, pruneDismissed, SHADOW, STACK_GAP, type FlyBox } from "./flyout";
import type { Session, State } from "./model";
import { STRIP_MAX } from "./strip";

function s(id: string, state: State, stateSince: number, extra: Partial<Session> = {}): Session {
  return { id, pid: 1, name: id, cwd: `c:/p/${id}`, kind: "interactive", entrypoint: "cli", state, stateSince, task: "", subagents: 0, alive: true, ...extra };
}

describe("flyWants", () => {
  it("flies out only waiting sessions, in strip order with their cell", () => {
    const sessions = [s("a", "working", 100), s("b", "waiting", 10), s("c", "waiting", 50), s("d", "error", 70)];
    expect(flyWants(sessions, true, new Map())).toEqual([
      { id: "c", slot: 1 },
      { id: "b", slot: 2 },
    ]);
  });

  it("flies nothing when the strip is not showing (panel, peek, hidden)", () => {
    expect(flyWants([s("b", "waiting", 10)], false, new Map())).toEqual([]);
  });

  it("skips ended sessions and ones dismissed in this waiting spell", () => {
    const sessions = [s("a", "waiting", 10, { alive: false }), s("b", "waiting", 20), s("c", "waiting", 30)];
    expect(flyWants(sessions, true, new Map([["b", 20]])).map((w) => w.id)).toEqual(["c"]);
    // Waiting again later is a new spell.
    expect(flyWants(sessions, true, new Map([["b", 5]])).map((w) => w.id)).toEqual(["c", "b"]);
  });

  it("maps a session behind the +N tile to the tile", () => {
    const busy = Array.from({ length: STRIP_MAX + 2 }, (_, i) => s(`w${i}`, "waiting", 1000 - i));
    const wants = flyWants(busy, true, new Map());
    expect(wants).toHaveLength(STRIP_MAX + 2);
    expect(wants.slice(STRIP_MAX - 1).map((w) => w.slot)).toEqual([STRIP_MAX - 1, STRIP_MAX, STRIP_MAX]);
  });
});

describe("pruneDismissed", () => {
  it("forgets a dismissal once the session leaves waiting or is gone", () => {
    const d = new Map([
      ["a", 10],
      ["b", 20],
      ["c", 30],
    ]);
    pruneDismissed(d, [s("a", "waiting", 10), s("b", "thinking", 25)]);
    expect([...d.keys()]).toEqual(["a"]);
  });
});

const box = (id: string, at: number, h = 90, w = 306): FlyBox => ({ id, w, h, at });
const zone = { w: 316, h: 1000 };

describe("layOut", () => {
  it("sits a column's flyout level with its peep, away from the strip", () => {
    const [left] = layOut("left", [box("a", 200)], zone);
    expect(left).toEqual({ id: "a", x: zone.w - GAP - 306, y: 200 - (90 - SHADOW) / 2, tail: (90 - SHADOW) / 2 });
    const [right] = layOut("right", [box("a", 200)], zone);
    expect(right.x).toBe(GAP);
  });

  it("stacks flyouts of neighbouring peeps in strip order without overlap", () => {
    const [a, b] = layOut("left", [box("a", 100), box("b", 158)], zone);
    expect(b.y).toBe(a.y + 90 + STACK_GAP);
    // Its peep is above the pushed-down card, so the tail holds at the top corner.
    expect(b.tail).toBe(12);
  });

  it("lifts a stack that would run off the bottom of the zone", () => {
    const placed = layOut("right", [box("a", 960), box("b", 980)], zone);
    const last = placed[1];
    expect(last.y + 90).toBe(zone.h);
    expect(placed[0].y).toBe(last.y - 90 - STACK_GAP);
  });

  it("stacks off a top-edge row downward, each under its own peep, the first pointing up", () => {
    const placed = layOut("down", [box("a", 600), box("b", 650)], { w: 2560, h: 1300 });
    expect(placed[0]).toEqual({ id: "a", x: 600 - 150, y: GAP, tail: 150 });
    expect(placed[1]).toMatchObject({ x: 650 - 150, y: GAP + 90 + STACK_GAP, tail: null });
  });

  it("stacks off a bottom-edge row upward and keeps cards inside the zone", () => {
    const placed = layOut("up", [box("a", 20), box("b", 70)], { w: 2560, h: 1300 });
    expect(placed[0]).toMatchObject({ x: 0, y: 1300 - GAP - 90, tail: 20 });
    expect(placed[1].y).toBe(placed[0].y - STACK_GAP - 90);
  });
});
