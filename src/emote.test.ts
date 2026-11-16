import { describe, expect, it } from "vitest";
import { clashLinks, clashTags, clashText, emoteFor, MOOD_EMOTE, nudgeFor, nudgeText } from "./emote";
import type { Session } from "./model";
import { toRowView } from "./rowView";

const base: Session = { id: "s", pid: 1, name: "n", cwd: "C:/x", kind: "interactive", entrypoint: "cli", state: "thinking", stateSince: 0, task: "", subagents: 0, alive: true };

describe("emoteFor", () => {
  it("maps each mood to its emote-sheet row", () => {
    expect(MOOD_EMOTE).toEqual({ startled: "startled", cheer: "sparkle", greet: "wave", tweet: "tweet", worry: "worry", yawn: "yawn", sweat: "sweat", nap: "zzz" });
    expect(emoteFor({ ...base, mood: "nap" }, "idle")).toBe("zzz");
    expect(emoteFor(base, "thinking")).toBeNull();
  });

  it("hides the emote while an attention state holds the bubble, and once the session closed", () => {
    for (const state of ["waiting", "error", "stuck"] as const) expect(emoteFor({ ...base, mood: "worry" }, state)).toBeNull();
    expect(emoteFor({ ...base, mood: "worry", alive: false }, "idle")).toBeNull();
    expect(toRowView({ ...base, state: "error", mood: "worry" }).emote).toBeNull();
    expect(toRowView({ ...base, mood: "greet" }).emote).toBe("wave");
    expect(emoteFor({ ...base, mood: "tweet" }, "waiting")).toBeNull();
    expect(emoteFor({ ...base, mood: "tweet" }, "idle")).toBe("tweet");
  });
});

describe("clash", () => {
  const startled = (id: string, clash: string, extra: Partial<Session> = {}): Session => ({ ...base, id, mood: "startled", clash, ...extra });
  const css = "F:\\p\\src\\shell.css";

  it("tags each clashed file A, B in list order and names the file", () => {
    const list = [startled("a", css), startled("b", "F:/p/x.ts"), startled("c", css), base];
    const tags = clashTags(list);
    expect(tags.get("a")).toEqual({ tag: "A", file: "shell.css", count: 2 });
    expect(tags.get("b")).toEqual({ tag: "B", file: "x.ts", count: 1 });
    expect(tags.get("c")?.tag).toBe("A");
    expect(tags.has("s")).toBe(false);
    expect(clashText({ tag: "A", file: "shell.css", count: 2 })).toBe("Both in shell.css");
    expect(clashText({ tag: "A", file: "shell.css", count: 3 })).toBe("All 3 in shell.css");
  });

  it("brackets only rows that already sit together", () => {
    const apart = [startled("a", css), base, startled("c", css)];
    expect(clashLinks(apart, clashTags(apart))).toEqual([false, false, false]);
    const together = [base, startled("a", css), startled("c", css)];
    expect(clashLinks(together, clashTags(together))).toEqual([false, true, false]);
  });

  it("an attention state keeps its bubble and drops out of the tags and the bracket", () => {
    const list = [startled("a", css), startled("b", css, { state: "waiting" })];
    const tags = clashTags(list);
    expect(tags.has("b")).toBe(false);
    expect(tags.get("a")?.count).toBe(2);
    expect(clashLinks(list, tags)).toEqual([false, false]);
    expect(toRowView(list[1], undefined, 0, false, tags.get("b")).bubble.bolt).toBeUndefined();
  });

  it("puts the bolt tag, file name and startled emote in the bubble without the attention fill", () => {
    const s = startled("a", css, { state: "working" });
    const v = toRowView(s, undefined, 0, false, clashTags([s, startled("b", css)]).get("a"));
    expect(v).toMatchObject({ emote: "startled", attention: false, say: "Both in shell.css", bubble: { text: "Both in shell.css", bolt: "A" } });
    expect(toRowView(s).bubble.bolt).toBeUndefined();
  });
});

describe("nudgeFor", () => {
  const tired = { ...base, mood: "yawn" as const, contextPct: 82 };

  it("nudges on the yawn, snoozes until 95%, then nudges once more", () => {
    expect(nudgeFor(tired, "thinking", false)).toBe("first");
    expect(nudgeFor(tired, "thinking", true)).toBeNull();
    expect(nudgeFor({ ...tired, contextPct: 95 }, "thinking", true)).toBe("last");
    expect(nudgeFor({ ...tired, mood: "sweat" }, "thinking", false)).toBeNull();
    expect(nudgeFor(tired, "waiting", false)).toBeNull();
  });

  it("puts the nudge copy in the bubble", () => {
    expect(nudgeText("first", 82.4)).toBe("Getting sleepy. I'm 82% full.");
    expect(toRowView(tired)).toMatchObject({ nudge: "first", say: "Getting sleepy. I'm 82% full.", emote: "yawn" });
    expect(toRowView(tired, undefined, 0, true)).toMatchObject({ nudge: null, say: "Hmm… thinking it over." });
    expect(toRowView({ ...tired, contextPct: 97 }).say).toBe("Nearly out of room. Pack up now and nothing gets lost.");
  });
});
