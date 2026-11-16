import { describe, expect, it, vi } from "vitest";
import { ACCESSORIES, identityFor, pickerAccessories } from "./identity";
import { createLookStore, cwdKey, pick, resolveLook } from "./look";
import type { Session } from "./model";

const base = { color: "blue", accessory: "none" } as const;
const session = (cwd: string) => ({ id: cwd, cwd }) as Session;

describe("cwdKey", () => {
  it("is the cwd hash as 8 hex digits, the same for every spelling (look.rs pins the same value)", () => {
    expect(cwdKey("C:\\Users\\Dev\\Projects\\AgentView")).toBe("62cc5ae9");
    expect(cwdKey("c:/users/dev/projects/agentview/")).toBe("62cc5ae9");
    expect(cwdKey("")).toBe("811c9dc5");
  });
});

describe("pickerAccessories", () => {
  it("is the free 14, then unlocked ones once each", () => {
    expect(pickerAccessories()).toEqual([...ACCESSORIES]);
    expect(pickerAccessories(["packed", "crown", "packed", "ten"])).toEqual([...ACCESSORIES, "packed", "ten"]);
  });
});

describe("resolveLook", () => {
  it("keeps the hashed look with no override", () => {
    expect(resolveLook(base, undefined)).toEqual(base);
  });

  it("applies each valid field and keeps the hashed one for a missing field", () => {
    expect(resolveLook(base, { color: "pink" })).toEqual({ color: "pink", accessory: "none" });
    expect(resolveLook(base, { accessory: "wizard" })).toEqual({ color: "blue", accessory: "wizard" });
  });

  it("ignores a colour off the palette and an accessory not offered", () => {
    expect(resolveLook(base, { color: "red", accessory: "packed" })).toEqual(base);
    expect(resolveLook(base, { accessory: "packed" }, pickerAccessories(["packed"]))).toEqual({ color: "blue", accessory: "packed" });
  });
});

describe("pick", () => {
  it("drops fields that equal the hashed look, so picking it back is a reset", () => {
    expect(pick(base, undefined, "color", "green")).toEqual({ color: "green" });
    expect(pick(base, { color: "green" }, "accessory", "crown")).toEqual({ color: "green", accessory: "crown" });
    expect(pick(base, { color: "green" }, "color", "blue")).toBeNull();
  });
});

describe("createLookStore", () => {
  it("applies a pick at once, keeps what the backend returns, and reset returns to the hash", async () => {
    const cwd = "C:/demo/app";
    const saved: Record<string, object> = {};
    const save = vi.fn(async (key: string, look: object | null) => {
      if (look) saved[key] = look;
      else delete saved[key];
      return { ...saved };
    });
    const changed = vi.fn();
    const store = createLookStore((s) => identityFor(s.cwd), save, changed);
    const s = session(cwd);
    const hashed = identityFor(cwd);
    const other = hashed.color === "pink" ? "green" : "pink";

    store.set(s, { color: other });
    expect(store.identityOf(s).color).toBe(other);
    expect(store.forCwd(cwd).color).toBe(other);
    await Promise.resolve();
    expect(save).toHaveBeenCalledWith(cwdKey(cwd), { color: other });

    store.set(s, null);
    expect(store.identityOf(s)).toEqual(hashed);
    expect(store.lookOf(s)).toBeUndefined();
    expect(changed).toHaveBeenCalled();
  });

  it("rolls back when the save fails", async () => {
    const err = vi.spyOn(console, "error").mockImplementation(() => {});
    const store = createLookStore((s) => identityFor(s.cwd), () => Promise.reject(new Error("disk")), () => {});
    const s = session("C:/demo/app");
    store.set(s, { accessory: "crown" });
    expect(store.identityOf(s).accessory).toBe("crown");
    await new Promise((r) => setTimeout(r, 0));
    expect(store.lookOf(s)).toBeUndefined();
    err.mockRestore();
  });
});
