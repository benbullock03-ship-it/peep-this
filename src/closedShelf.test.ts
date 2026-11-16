import { describe, expect, it } from "vitest";
import { agoText, closedFlags, closedOrder, endLabel, moreText, resumeCommand } from "./closedShelf";
import type { ClosedSession } from "./model";

const NO_FLAGS = { uncommitted: false, ticketDoing: false, noPackup: false, attention: false };
const c = (id: string, endedAt: number, extra: Partial<ClosedSession> = {}): ClosedSession => ({
  sessionId: id,
  name: id,
  cwd: "C:/work/demo",
  bubble: "Done",
  state: "success",
  endedAt,
  flags: NO_FLAGS,
  ...extra,
});

describe("closedOrder", () => {
  it("puts the newest first whatever the flags, and drops sessions that are live again", () => {
    const flagged = c("b", 100, { flags: { ...NO_FLAGS, uncommitted: true } });
    const got = closedOrder([c("a", 300), flagged, c("c", 200), c("back", 400)], new Set(["back"]));
    expect(got.map((x) => x.sessionId)).toEqual(["a", "c", "b"]);
  });
});

describe("closedFlags", () => {
  it("shows uncommitted, the ticket left doing, then no Pack Up", () => {
    const all = { uncommitted: true, ticketDoing: true, noPackup: true, attention: true };
    expect(closedFlags(c("a", 0, { ticket: { id: "P-12" }, flags: all }))).toEqual(["uncommitted", "P-12 left doing", "no Pack Up"]);
  });

  it("never flags the attention end, and skips left doing without a ticket", () => {
    expect(closedFlags(c("a", 0, { state: "waiting", flags: { ...NO_FLAGS, attention: true, ticketDoing: true } }))).toEqual([]);
  });
});

describe("endLabel", () => {
  it("says ended for attention states and the state otherwise", () => {
    expect(endLabel(c("a", 0, { state: "waiting" }))).toBe("ended waiting");
    expect(endLabel(c("a", 0, { state: "error" }))).toBe("ended error");
    expect(endLabel(c("a", 0, { state: "success" }))).toBe("success");
  });
});

describe("text", () => {
  it("formats time since end in minutes, then hours", () => {
    expect([agoText(30_000), agoText(14 * 60_000), agoText(61 * 60_000), agoText(7.9 * 3_600_000)]).toEqual(["now", "14m", "1h", "7h"]);
  });

  it("drops zero parts of the footer", () => {
    expect(moreText(15, 3)).toBe("15 more below · 3 to check");
    expect(moreText(2, 0)).toBe("2 more below");
    expect(moreText(0, 0)).toBe("");
  });

  it("builds the resume command", () => {
    expect(resumeCommand("3f9a2c1e")).toBe("claude --resume 3f9a2c1e");
  });
});
