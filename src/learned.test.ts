// @ts-expect-error type error without @types/node package
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { LESSONS, learnedView, PATTERN_IDS, rhythmSummary, SKILL_WHY, TOPICS, type Rhythm } from "./learned";

const none: Rhythm = { packups: { count: 0, first: null }, tickets: { count: 0, first: null }, approvals: { count: 0, first: null } };

describe("learnedView", () => {
  it("shows only seen lessons, grouped in topic order, and counts the rest", () => {
    const v = learnedView({ tips: ["crowded", "helper"], skills: ["ducks"], patterns: ["god-file", "not-a-pattern"], rhythm: none });
    expect(v.groups.map((g) => g.topic)).toEqual(["context", "rhythm", "working with sessions", "architecture"]);
    expect(v.groups.flatMap((g) => g.lessons.map((l) => l.id))).toEqual(["crowded", "ducks", "helper", "god-file"]);
    expect(v.remaining).toBe(LESSONS.length - 4);
    expect(learnedView({ tips: [], skills: [], patterns: [], rhythm: none })).toEqual({ groups: [], remaining: LESSONS.length });
  });

  it("gives every lesson a topic, an ELI5 and a source", () => {
    for (const l of LESSONS) {
      expect(TOPICS).toContain(l.topic);
      expect(l.eli5.length).toBeGreaterThan(10);
      expect(l.source.label).toBeTruthy();
      if (l.source.url) expect(l.source.url).toMatch(/^https:\/\//);
    }
  });
});

describe("rhythmSummary", () => {
  it("lists the three counts with the first-use day, and no day for a habit not yet used", () => {
    const noon = (d: string) => Date.parse(`${d}T12:00:00Z`);
    const r: Rhythm = { ...none, packups: { count: 3, first: noon("2026-11-12") }, approvals: { count: 12, first: noon("2026-11-09") } };
    expect(rhythmSummary(r)).toBe(
      [
        "Peep This rhythm",
        "Sessions with a Pack Up: 3, first 2026-11-12",
        "Tickets closed through /peep:ticket: 0",
        "Approvals from the widget: 12, first 2026-11-09",
      ].join("\n"),
    );
  });
});

describe("learned catalogue stays in step with the plugin", () => {
  it("each skill's Why line matches its SKILL.md", () => {
    for (const [id, why] of Object.entries(SKILL_WHY)) {
      const md: string = readFileSync(new URL(`../plugin/skills/${id}/SKILL.md`, import.meta.url), "utf8");
      expect(md, id).toContain(`Why: ${why}`);
    }
  });

  it("the patterns are the ids in patterns.md", () => {
    const md: string = readFileSync(new URL("../plugin/references/patterns.md", import.meta.url), "utf8");
    const ids = [...md.matchAll(/^\| `([a-z-]+)` \|/gm)].map((m) => m[1]);
    expect([...PATTERN_IDS].sort()).toEqual(ids.sort());
  });
});
