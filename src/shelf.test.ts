import { describe, expect, it } from "vitest";
import type { RepoTicket } from "./model";
import { CHIP, parkedLine, shelfOrder } from "./shelf";

const t = (id: string, status: RepoTicket["status"], extra: Partial<RepoTicket> = {}): RepoTicket => ({ id, title: id, status, ...extra });

describe("shelfOrder", () => {
  it("sorts blocked, active, review, todo, parked and keeps file order within a status", () => {
    const got = shelfOrder([t("A", "deferred"), t("B", "open"), t("C", "done"), t("D", "doing"), t("E", "open"), t("F", "blocked")]);
    expect(got.map((x) => x.id)).toEqual(["F", "D", "C", "B", "E", "A"]);
  });
});

describe("CHIP", () => {
  it("shows the design's names for the file statuses", () => {
    expect(Object.fromEntries(Object.entries(CHIP).map(([k, v]) => [k, v.label]))).toEqual({
      open: "todo",
      doing: "active",
      done: "review",
      blocked: "blocked",
      deferred: "parked",
    });
  });
});

describe("parkedLine", () => {
  it("reads reason and wake-up without doubled full stops", () => {
    expect(parkedLine(t("A", "deferred", { reason: "waiting on the emote sheet.", wake: "AV-31 is done" }))).toBe(
      "Parked: waiting on the emote sheet. Wakes when AV-31 is done.",
    );
  });

  it("labels blocked tickets and skips missing parts", () => {
    expect(parkedLine(t("A", "blocked", { wake: "2026-12-01 passes" }))).toBe("Wakes when 2026-12-01 passes.");
    expect(parkedLine(t("A", "blocked", { reason: "no key" }))).toBe("Blocked: no key.");
  });

  it("is empty for active tickets", () => {
    expect(parkedLine(t("A", "open", { reason: "x" }))).toBe("");
  });
});
