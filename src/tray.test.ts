import { describe, expect, it } from "vitest";
import { FLOPPIES } from "./bubble";
import { hintText, jumpCommand, TRAY } from "./tray";

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
