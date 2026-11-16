// @ts-expect-error type error without @types/node package
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { FLOPPIES } from "./bubble";
import { hintText, jumpCommand, TRAY, TRAY_SHOWN } from "./tray";

describe("jumpCommand", () => {
  it("copies the bare skill command", () => {
    expect(jumpCommand("packup")).toBe("/peep:packup");
  });

  it("appends a trimmed argument such as a ticket id", () => {
    expect(jumpCommand("ticket", " AV-34 ")).toBe("/peep:ticket AV-34");
  });

  it("drops an empty argument", () => {
    expect(jumpCommand("ticket", "  ")).toBe("/peep:ticket");
  });
});

describe("hintText", () => {
  it("keeps the instruction for the first three Jumps", () => {
    expect(hintText("/peep:qt", 3)).toBe("Copied /peep:qt · now paste it in the terminal");
    expect(hintText("/peep:qt", 4)).toBe("Copied /peep:qt");
  });
});

it("every name in PEEP_SKILLS maps to its own floppy sprite", () => {
  const rs: string = readFileSync(new URL("../src-tauri/src/state.rs", import.meta.url), "utf8");
  const list = rs.match(/PEEP_SKILLS: \[&str; \d+\] =\s*\[([^\]]*)\]/)![1];
  const skills = [...list.matchAll(/"([a-z]+)"/g)].map((m) => m[1]);
  expect(skills).toHaveLength(11);
  expect(Object.keys(FLOPPIES).sort()).toEqual([...skills].sort());
  expect(new Set(Object.values(FLOPPIES).map((f) => f.col)).size).toBe(skills.length);
});

it("orders the tray Core then Extras, six shown and the rest behind +N", () => {
  expect(TRAY.slice(0, TRAY_SHOWN).map((f) => f.skill)).toEqual(["walk", "qt", "ticket", "shoot", "packup", "ducks"]);
  expect(TRAY.slice(TRAY_SHOWN).map((f) => f.skill)).toEqual(["sneak", "goose", "coderot", "bigpeep"]);
});

it("tweet seats on the plank but has no tray cell", () => {
  expect(FLOPPIES.tweet).toBeDefined();
  expect(TRAY.some((f) => f.skill === "tweet")).toBe(false);
});

it("each hover card's use-when line is the tour's", () => {
  const tour: string = readFileSync(new URL("../plugin/skills/walk/references/tour.md", import.meta.url), "utf8");
  for (const f of TRAY.filter((f) => f.skill !== "walk")) {
    const line = tour.match(new RegExp(`/peep:${f.skill}\\s+(use .+)$`, "m"))![1];
    expect(f.when, f.skill).toBe(`U${line.slice(1)}.`);
  }
});
