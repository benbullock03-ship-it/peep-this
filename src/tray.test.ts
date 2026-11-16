import { describe, expect, it } from "vitest";
import { FLOPPIES } from "./bubble";
import { hintText, jumpCommand, TRAY, TRAY_SHOWN } from "./tray";
import { UNLOCKS } from "./unlocks";

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

it("every unlock rule names a skill the tray or the tour knows", () => {
  const known = new Set(TRAY.map((f) => f.skill));
  for (const s of Object.keys(UNLOCKS)) expect(known.has(s)).toBe(true);
});

it("every tray floppy has sprite art", () => {
  for (const f of TRAY) expect(FLOPPIES[f.skill]).toBeDefined();
});

it("shows the six spec floppies and puts the rest behind +N", () => {
  expect(TRAY.map((f) => f.skill)).toEqual(["ducks", "qt", "ticket", "shoot", "packup", "coderot", "walk", "sneak", "scrub", "bigpeep", "tweet", "goose"]);
  expect(TRAY.slice(TRAY_SHOWN).map((f) => jumpCommand(f.skill))).toEqual([
    "/peep:walk",
    "/peep:sneak",
    "/peep:scrub",
    "/peep:bigpeep",
    "/peep:tweet",
    "/peep:goose",
  ]);
  expect(new Set(TRAY.map((f) => FLOPPIES[f.skill].col)).size).toBe(TRAY.length);
});
