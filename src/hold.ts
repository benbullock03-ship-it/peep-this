import { createHold, HOLD_MS } from "./approval";

/** Press-and-hold button (AV-22, AV-97): pointer or Space/Enter held for HOLD_MS fires `done`; any release first cancels. */
export function bindHold(btn: HTMLElement, done: () => void, canStart: () => boolean = () => true) {
  btn.style.setProperty("--hold-ms", `${HOLD_MS}ms`);
  const hold = createHold(() => {
    btn.classList.remove("holding");
    done();
  });
  const start = () => {
    if (!canStart()) return;
    btn.classList.add("holding");
    hold.start();
  };
  const stop = () => {
    btn.classList.remove("holding");
    hold.cancel();
  };
  btn.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    btn.setPointerCapture(e.pointerId);
    start();
  });
  btn.addEventListener("pointerup", stop);
  btn.addEventListener("pointercancel", stop);
  btn.addEventListener("lostpointercapture", stop);
  btn.addEventListener("keydown", (e) => {
    if ((e.key === " " || e.key === "Enter") && !e.repeat) start();
    if (e.key === " " || e.key === "Enter") e.preventDefault();
  });
  btn.addEventListener("keyup", stop);
  btn.addEventListener("blur", stop);
  return { start, stop };
}
