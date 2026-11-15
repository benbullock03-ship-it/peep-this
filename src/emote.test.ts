import { describe, expect, it } from "vitest";
import { emoteFor, MOOD_EMOTE, nudgeFor, nudgeText } from "./emote";
import type { Session } from "./model";
import { toRowView } from "./rowView";

const base: Session = { id: "s", pid: 1, name: "n", cwd: "C:/x", kind: "interactive", entrypoint: "cli", state: "thinking", stateSince: 0, task: "", subagents: 0, alive: true };

describe("emoteFor", () => {
  it("maps each mood to its emote-sheet row", () => {
    expect(MOOD_EMOTE).toEqual({ cheer: "sparkle", greet: "wave", worry: "worry", yawn: "yawn", sweat: "sweat", nap: "zzz" });
    expect(emoteFor({ ...base, mood: "nap" }, "idle")).toBe("zzz");
    expect(emoteFor(base, "thinking")).toBeNull();
  });

  it("hides the emote while an attention state holds the bubble, and once the session closed", () => {
    for (const state of ["waiting", "error", "stuck"] as const) expect(emoteFor({ ...base, mood: "worry" }, state)).toBeNull();
    expect(emoteFor({ ...base, mood: "worry", alive: false }, "idle")).toBeNull();
    expect(toRowView({ ...base, state: "error", mood: "worry" }).emote).toBeNull();
    expect(toRowView({ ...base, mood: "greet" }).emote).toBe("wave");
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
