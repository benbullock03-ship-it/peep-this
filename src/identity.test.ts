import { describe, expect, it } from "vitest";
import { ACCESSORIES, COLORS, fnv1a, identityFor, normaliseCwd } from "./identity";

describe("fnv1a", () => {
  it("matches the published 32-bit FNV-1a vectors", () => {
    expect(fnv1a("")).toBe(0x811c9dc5);
    expect(fnv1a("a")).toBe(0xe40c292c);
    expect(fnv1a("foobar")).toBe(0xbf9cf968);
  });
});

describe("normaliseCwd", () => {
  it("lower-cases, uses forward slashes and drops trailing slashes", () => {
    expect(normaliseCwd("C:\\Users\\Dev\\Projects\\AgentView\\")).toBe("c:/users/dev/projects/agentview");
    expect(normaliseCwd("/home/dev/app//")).toBe("/home/dev/app");
  });
});

describe("identityFor", () => {
  it("gives the same identity for every spelling of one cwd", () => {
    const variants = [
      "C:\\Users\\Dev\\Projects\\AgentView",
      "c:\\users\\dev\\projects\\agentview\\",
      "C:/Users/Dev/Projects/AgentView",
      "c:/users/dev/projects/agentview/",
    ];
    const first = identityFor(variants[0]);
    for (const v of variants) expect(identityFor(v)).toEqual(first);
  });

  it("pins known outputs so the mapping never drifts", () => {
    expect(identityFor("C:\\Users\\Dev\\Projects\\AgentView")).toEqual({ color: "yellow", accessory: "beanie" });
    expect(identityFor("F:\\Projects\\agentview widget")).toEqual({ color: "blue", accessory: "none" });
    expect(identityFor("/home/dev/app")).toEqual({ color: "yellow", accessory: "shades" });
    expect(identityFor("")).toEqual({ color: "pink", accessory: "halo" });
  });

  it("only returns names peep.js supports and reaches all of them", () => {
    const colors = new Set<string>();
    const accs = new Set<string>();
    for (let i = 0; i < 500; i++) {
      const id = identityFor(`c:/projects/p${i}`);
      colors.add(id.color);
      accs.add(id.accessory);
    }
    expect([...colors].sort()).toEqual([...COLORS].sort());
    expect([...accs].sort()).toEqual([...ACCESSORIES].sort());
  });
});
