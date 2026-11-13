import { describe, expect, it } from "vitest";
import type { Session, State } from "./model";
import { sortSessions } from "./sort";

function s(id: string, state: State, stateSince: number): Session {
  return { id, pid: 1, name: id, cwd: "c:/p", kind: "interactive", entrypoint: "cli", state, stateSince, task: "", subagents: 0, alive: true };
}

const ids = (xs: Session[]) => xs.map((x) => x.id);

describe("sortSessions", () => {
  it("puts waiting sessions first, then newest state change first", () => {
    const out = sortSessions([s("a", "working", 100), s("b", "waiting", 10), s("c", "idle", 300), s("d", "waiting", 50)]);
    expect(ids(out)).toEqual(["d", "b", "c", "a"]);
  });

  it("orders ties by id so the list does not jitter", () => {
    expect(ids(sortSessions([s("y", "idle", 5), s("x", "idle", 5)]))).toEqual(["x", "y"]);
  });

  it("does not mutate its input", () => {
    const input = [s("a", "idle", 1), s("b", "waiting", 1)];
    sortSessions(input);
    expect(ids(input)).toEqual(["a", "b"]);
  });
});
