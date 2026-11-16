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

it("every tray floppy has sprite art", () => {
  for (const f of TRAY) expect(FLOPPIES[f.skill]).toBeDefined();
});

it("shows the six spec floppies and puts the phase 1 three behind +N", () => {
  expect(TRAY.map((f) => f.skill)).toEqual(["ducks", "qt", "ticket", "shoot", "packup", "coderot", "unpack", "walk", "reel"]);
  expect(TRAY.slice(TRAY_SHOWN).map((f) => jumpCommand(f.skill))).toEqual(["/peep:unpack", "/peep:walk", "/peep:reel"]);
  expect(new Set(TRAY.map((f) => FLOPPIES[f.skill].col)).size).toBe(TRAY.length);
});
