import { describe, expect, it } from "vitest";
import { entryAction, factsOf, filesLine, headline, sections } from "./digest";
import type { DigestEntry, DigestGroup } from "./model";

const entry = (name: string, group: DigestGroup, over: Partial<DigestEntry> = {}): DigestEntry => ({
  sessionId: `id-${name}`,
  name,
  group,
  alive: true,
  pending: false,
  tookMs: 0,
  files: [],
  fileCount: 0,
  commands: 0,
  released: 0,
  partial: false,
  ...over,
});

describe("digest card", () => {
  it("sections follow blocked, failed, finished, running and skip empty groups", () => {
    const got = sections([entry("r", "running"), entry("b", "blocked"), entry("f", "finished"), entry("b2", "blocked")]);
    expect(got.map((s) => [s.group, s.entries.map((e) => e.name)])).toEqual([
      ["blocked", ["b", "b2"]],
      ["finished", ["f"]],
      ["running", ["r"]],
    ]);
    expect(sections([])).toEqual([]);
  });

  it("facts read as time, files, commands and releases, floors marked when partial", () => {
    expect(factsOf(entry("a", "finished", { tookMs: 125_000 }))).toEqual(["2m 05s"]);
    const e = entry("a", "finished", { tookMs: 3_600_000, fileCount: 1, commands: 2, released: 1 });
    expect(factsOf(e)).toEqual(["1h 00m", "1 file changed", "2 commands", "1 approval went to the terminal"]);
    expect(factsOf({ ...e, partial: true, commands: 1 })).toEqual(["1h 00m", "1+ files changed", "1+ commands", "1 approval went to the terminal"]);
  });

  it("files show base names and count the unlisted ones", () => {
    expect(filesLine(entry("a", "finished"))).toBe("");
    expect(filesLine(entry("a", "finished", { files: ["src\\a.ts", "src/b/c.rs"], fileCount: 2 }))).toBe("a.ts, c.rs");
    expect(filesLine(entry("a", "finished", { files: ["x.ts"], fileCount: 10 }))).toBe("x.ts +9");
  });

  it("blocked on a held approval opens the inbox; ended sessions do nothing", () => {
    expect(entryAction(entry("a", "blocked", { pending: true }))).toBe("inbox");
    expect(entryAction(entry("a", "blocked"))).toBe("focus");
    expect(entryAction(entry("a", "running"))).toBe("focus");
    expect(entryAction(entry("a", "finished", { alive: false }))).toBe("none");
  });

  it("headline names the absence and the session count", () => {
    expect(headline(0, 42 * 60_000, [entry("a", "running")])).toBe("Away 42m 00s · 1 session");
    expect(headline(0, 61_000, [entry("a", "running"), entry("b", "failed")])).toBe("Away 1m 01s · 2 sessions");
  });
});
