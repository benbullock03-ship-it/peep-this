import { describe, expect, it } from "vitest";
import { barBlocks, barLevel } from "./bar";

describe("barBlocks", () => {
  it("lights nothing at zero or for non-finite values", () => {
    expect(barBlocks(0)).toBe(0);
    expect(barBlocks(-5)).toBe(0);
    expect(barBlocks(Number.NaN)).toBe(0);
  });

  it("lights one block for any value above zero", () => {
    expect(barBlocks(0.1)).toBe(1);
    expect(barBlocks(4)).toBe(1);
  });

  it("rounds to the nearest block", () => {
    expect(barBlocks(14)).toBe(1);
    expect(barBlocks(15)).toBe(2);
    expect(barBlocks(44)).toBe(4);
    expect(barBlocks(74)).toBe(7);
    expect(barBlocks(93)).toBe(9);
    expect(barBlocks(96)).toBe(10);
  });

  it("caps at ten blocks", () => {
    expect(barBlocks(100)).toBe(10);
    expect(barBlocks(250)).toBe(10);
  });
});

describe("barLevel", () => {
  it("is neutral below 70, amber from 70, red from 90", () => {
    expect(barLevel(0)).toBe("neutral");
    expect(barLevel(69.9)).toBe("neutral");
    expect(barLevel(70)).toBe("amber");
    expect(barLevel(89.9)).toBe("amber");
    expect(barLevel(90)).toBe("red");
    expect(barLevel(100)).toBe("red");
  });
});
