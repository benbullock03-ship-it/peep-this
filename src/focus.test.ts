import { describe, expect, it, vi } from "vitest";
import { requestFocus } from "./rowView";

describe("requestFocus", () => {
  it("does not flash when a window came forward", async () => {
    const flash = vi.fn();
    const onFocus = vi.fn(async () => true);
    await requestFocus(onFocus, "abc", flash);
    expect(onFocus).toHaveBeenCalledWith("abc");
    expect(flash).not.toHaveBeenCalled();
  });

  it("flashes when no window was found", async () => {
    const flash = vi.fn();
    await requestFocus(async () => false, "abc", flash);
    expect(flash).toHaveBeenCalledOnce();
  });

  it("flashes when the command fails", async () => {
    const flash = vi.fn();
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    await requestFocus(async () => Promise.reject(new Error("denied")), "abc", flash);
    expect(flash).toHaveBeenCalledOnce();
    spy.mockRestore();
  });
});
