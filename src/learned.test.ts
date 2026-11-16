// @ts-expect-error type error without @types/node package
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { LESSONS, learnedView, PATTERN_IDS, SKILL_WHY, TOPICS } from "./learned";

describe("learnedView", () => {
  it("shows only seen lessons, grouped in topic order, and counts the rest", () => {
    const v = learnedView({ tips: ["crowded", "helper"], skills: ["ducks"], patterns: ["god-file", "not-a-pattern"] });
    expect(v.groups.map((g) => g.topic)).toEqual(["context", "rhythm", "working with sessions", "architecture"]);
    expect(v.groups.flatMap((g) => g.lessons.map((l) => l.id))).toEqual(["crowded", "ducks", "helper", "god-file"]);
    expect(v.remaining).toBe(LESSONS.length - 4);
    expect(learnedView({ tips: [], skills: [], patterns: [] })).toEqual({ groups: [], remaining: LESSONS.length });
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
