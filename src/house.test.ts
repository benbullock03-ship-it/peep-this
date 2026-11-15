import { describe, expect, it } from "vitest";
import { flyWants } from "./flyout";
import { houseShown } from "./house";
import type { Session } from "./model";
import { stripView } from "./strip";

function s(id: string, extra: Partial<Session> = {}): Session {
  return { id, pid: 1, name: id, cwd: `c:/p/${id}`, kind: "interactive", entrypoint: "cli", state: "idle", stateSince: 1, task: "", subagents: 0, alive: true, ...extra };
}

describe("houseShown", () => {
  it("shows with no sessions and nothing fading", () => {
    expect(houseShown([])).toBe(true);
    expect(houseShown([], 0)).toBe(true);
  });

  it("leaves as soon as the first session arrives", () => {
    expect(houseShown([s("a")])).toBe(false);
  });

  it("stays away while an ended session is still in the snapshot", () => {
    expect(houseShown([s("a", { alive: false, state: "success" })])).toBe(false);
  });

  it("stays away until the last fading row is gone", () => {
    expect(houseShown([], 1)).toBe(false);
    expect(houseShown([], 2)).toBe(false);
  });
});

describe("the house peep is not counted", () => {
  it("gives an empty fleet no strip cells and no +N", () => {
    expect(stripView([])).toEqual({ cells: [], more: 0 });
  });

  it("never flies out", () => {
    expect(flyWants([], true, new Map())).toEqual([]);
  });
});
