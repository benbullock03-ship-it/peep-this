import { describe, expect, it } from "vitest";
import { bubbleFor, bubbleLine } from "./bubble";
import type { Session } from "./model";
import { toRowView } from "./rowView";

const live = { alive: true };
const ticket = { id: "AV-34", title: "chip contrast" };

describe("bubbleFor", () => {
  it("reads `ID: title · verb` with a ticket and a running skill", () => {
    const b = bubbleFor({ ...live, skill: "shoot", ticket }, "Running npm test…", false);
    expect(b).toEqual({ id: "AV-34", text: "chip contrast", verb: "shooting" });
    expect(bubbleLine(b)).toBe("AV-34: chip contrast · shooting");
    expect(bubbleLine(bubbleFor({ ...live, skill: "packup", ticket }, "", false))).toBe("AV-34: chip contrast · packing up");
  });

  it("keeps the say line without a ticket, and the verb only while a skill runs", () => {
    expect(bubbleLine(bubbleFor({ ...live, skill: "ducks" }, "Running git status…", false))).toBe("Running git status… · ducking");
    expect(bubbleFor(live, "All done!", false)).toEqual({ text: "All done!" });
    expect(bubbleLine(bubbleFor({ ...live, ticket }, "All done!", false))).toBe("AV-34: chip contrast");
  });

  it("keeps the id whole and lets the say line stand in for a missing title", () => {
    const long = "a title long enough that the bubble has to ellipsise it well before the verb";
    expect(bubbleFor({ ...live, skill: "ticket", ticket: { id: "AV-34", title: long } }, "", false)).toEqual({ id: "AV-34", text: long, verb: "ticketing" });
    expect(bubbleFor({ ...live, ticket: { id: "AV-7" } }, "Hmm… thinking it over.", false)).toEqual({ id: "AV-7", text: "Hmm… thinking it over.", verb: undefined });
  });

  it("lets an attention state take over: the ask replaces the title, the id becomes a tab, the verb goes", () => {
    expect(bubbleFor({ ...live, skill: "shoot", ticket }, "Can I run npm test?", true)).toEqual({ id: "AV-34", text: "Can I run npm test?", tab: true });
    expect(bubbleFor({ ...live, skill: "shoot" }, "Can I run npm test?", true)).toEqual({ text: "Can I run npm test?" });
  });

  it("puts a rate-limit warning in the verb slot, or the whole line with no ticket, never over an ask", () => {
    const limit = { key: "5h" as const, pct: 82 };
    expect(bubbleLine(bubbleFor({ ...live, skill: "ticket", ticket: { id: "AV-81", title: "rate-limit warning" } }, "", false, limit))).toBe("AV-81: rate-limit warning · 5h 82%");
    expect(bubbleFor({ ...live, skill: "shoot" }, "Running npm test…", false, limit)).toEqual({ text: "5h limit 82%" });
    expect(bubbleFor({ ...live, ticket }, "Can I run npm test?", true, limit)).toEqual({ id: "AV-34", text: "Can I run npm test?", tab: true });
  });

  it("says goodbye plainly once the session has closed", () => {
    expect(bubbleFor({ alive: false, skill: "shoot", ticket }, "Session closed. Bye!", false)).toEqual({ text: "Session closed. Bye!" });
  });
});

describe("toRowView floppy", () => {
  const base: Session = { id: "s", pid: 1, name: "n", cwd: "C:/x", kind: "interactive", entrypoint: "cli", state: "working", stateSince: 0, task: "", subagents: 0, alive: true };

  it("seats the skill's floppy, a blank one for an unknown skill, and none after Stop", () => {
    expect(toRowView({ ...base, skill: "packup" }).floppy).toBe(4);
    expect(toRowView({ ...base, skill: "mystery" }).floppy).toBe(6);
    expect(toRowView(base).floppy).toBeNull();
    expect(toRowView({ ...base, state: "waiting", skill: "shoot", ticket, task: "Needs a look" })).toMatchObject({ floppy: 3, say: "AV-34: Needs a look" });
  });

  it("shows the limit text only while the row sweats", () => {
    const limit = { key: "7d" as const, pct: 91 };
    expect(toRowView({ ...base, mood: "sweat" }, undefined, 0, false, undefined, limit).say).toBe("7d limit 91%");
    expect(toRowView({ ...base, mood: "greet" }, undefined, 0, false, undefined, limit).say).not.toContain("limit");
    expect(toRowView({ ...base, mood: "sweat", state: "waiting", task: "Needs a look" }, undefined, 0, false, undefined, limit).say).toBe("Needs a look");
  });
});
