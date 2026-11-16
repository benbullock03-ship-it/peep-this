import { describe, expect, it } from "vitest";
import { lockHint, type Unlock } from "./unlocks";

const fresh: Unlock = { ticketRuns: 0, closedUi: false, test: false, url: false };
const repo = (closed: number, u: Partial<Unlock> = {}) => ({ arc: { closed }, unlock: { ...fresh, ...u } });

describe("lockHint", () => {
  it("locks every ruled Extra in a fresh repo", () => {
    for (const s of ["bigpeep", "coderot", "goose", "sneak", "scrub", "shoot"]) expect(lockHint(s, repo(0))).not.toBeNull();
    expect(lockHint("bigpeep", repo(0))).toBe("unlocks after 5 tickets");
  });

  it("never locks the Core or unruled skills", () => {
    for (const s of ["ducks", "qt", "ticket", "packup", "walk", "tweet"]) expect(lockHint(s, repo(0))).toBeNull();
  });

  it("unlocks each Extra at its threshold", () => {
    expect(lockHint("bigpeep", repo(0, { ticketRuns: 4 }))).not.toBeNull();
    expect(lockHint("bigpeep", repo(0, { ticketRuns: 5 }))).toBeNull();
    expect(lockHint("coderot", repo(1))).not.toBeNull();
    expect(lockHint("coderot", repo(0, { test: true }))).not.toBeNull();
    expect(lockHint("coderot", repo(1, { test: true }))).toBeNull();
    expect(lockHint("goose", repo(2))).not.toBeNull();
    expect(lockHint("goose", repo(3))).toBeNull();
    expect(lockHint("sneak", repo(1, { closedUi: true }))).toBeNull();
    expect(lockHint("scrub", repo(1, { closedUi: true }))).toBeNull();
    expect(lockHint("shoot", repo(0, { url: true }))).toBeNull();
  });

  it("locks nothing without a repo or its unlock facts", () => {
    expect(lockHint("bigpeep", undefined)).toBeNull();
    expect(lockHint("bigpeep", { arc: { closed: 0 } })).toBeNull();
  });
});
