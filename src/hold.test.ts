import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { bindHold } from "./hold";

function fakeButton() {
  const classes = new Set<string>();
  const t = Object.assign(new EventTarget(), {
    classList: { add: (c: string) => classes.add(c), remove: (c: string) => classes.delete(c) },
    style: { setProperty: () => {} },
    setPointerCapture: () => {},
  });
  return { btn: t as unknown as HTMLElement, classes };
}

const fire = (btn: HTMLElement, type: string, init: Record<string, unknown> = {}) =>
  btn.dispatchEvent(Object.assign(new Event(type), init));

describe("bindHold", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("fires after a full 1s press and clears the fill", () => {
    const { btn, classes } = fakeButton();
    const done = vi.fn();
    bindHold(btn, done);
    fire(btn, "pointerdown", { button: 0, pointerId: 1 });
    expect(classes.has("holding")).toBe(true);
    vi.advanceTimersByTime(999);
    expect(done).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(done).toHaveBeenCalledTimes(1);
    expect(classes.has("holding")).toBe(false);
  });

  it("cancels on early release, so a short click does nothing", () => {
    const { btn, classes } = fakeButton();
    const done = vi.fn();
    bindHold(btn, done);
    fire(btn, "pointerdown", { button: 0, pointerId: 1 });
    vi.advanceTimersByTime(500);
    fire(btn, "pointerup");
    expect(classes.has("holding")).toBe(false);
    vi.advanceTimersByTime(2000);
    expect(done).not.toHaveBeenCalled();
  });

  it("holds from the keyboard and ignores other mouse buttons and a refused start", () => {
    const { btn } = fakeButton();
    const done = vi.fn();
    let allowed = false;
    bindHold(btn, done, () => allowed);
    fire(btn, "pointerdown", { button: 0, pointerId: 1 });
    vi.advanceTimersByTime(1000);
    allowed = true;
    fire(btn, "pointerdown", { button: 2, pointerId: 1 });
    vi.advanceTimersByTime(1000);
    expect(done).not.toHaveBeenCalled();
    fire(btn, "keydown", { key: " ", repeat: false });
    vi.advanceTimersByTime(1000);
    expect(done).toHaveBeenCalledTimes(1);
  });
});
