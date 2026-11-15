/** AV-39: the strip/panel switch, animated inside a window held on the union of both shapes (spec §12.3). */

export type Box = { x: number; y: number; w: number; h: number };
/** Mirrors `FoldView` in src-tauri/src/window.rs: CSS px, window rects with the shadow included. */
export type FoldGeometry = { win: [number, number]; from: Box; to: Box; covered?: boolean };

/** Owner's choice 2026-11-15: 12 steps over 240ms, an exception to the 4 to 6 frame / 160ms rule. */
export const FOLD_MS = 240;
export const FOLD_STEPS = 12;

/** Matches the `.ps` hard shadow and window.rs SHADOW. */
const SHADOW = 6;
/** The region runs this many steps ahead of a growing card, so IPC lag never clips it. */
export const REGION_LEAD = 3;

export function boxOf(r: readonly number[]): Box {
  return { x: r[0], y: r[1], w: r[2], h: r[3] };
}

export function lerpBox(a: Box, b: Box, p: number): Box {
  const l = (m: number, n: number) => m + (n - m) * p;
  return { x: l(a.x, b.x), y: l(a.y, b.y), w: l(a.w, b.w), h: l(a.h, b.h) };
}

export function bounds(...boxes: Box[]): Box {
  const x = Math.min(...boxes.map((b) => b.x));
  const y = Math.min(...boxes.map((b) => b.y));
  return { x, y, w: Math.max(...boxes.map((b) => b.x + b.w)) - x, h: Math.max(...boxes.map((b) => b.y + b.h)) - y };
}

const cardOf = (win: Box): Box => ({ x: win.x, y: win.y, w: win.w - SHADOW, h: win.h - SHADOW });

/** Progress `elapsed` ms in. Each step shows the end of its interval, so the first one lands on the click. */
export function progressAt(elapsed: number): number {
  if (elapsed >= FOLD_MS) return 1;
  return (Math.floor((Math.max(elapsed, 0) / FOLD_MS) * FOLD_STEPS) + 1) / FOLD_STEPS;
}

/** Cross-fade over time fraction `u`, in quarters: the outgoing shape is gone by 40%, the incoming one arrives from 20% to 70%. */
export function fadeAt(u: number): { out: number; in: number } {
  const q = (v: number) => Math.floor(Math.min(1, Math.max(0, v)) * 4) / 4;
  return { out: 1 - q(u / 0.4), in: q((u - 0.2) / 0.5) };
}

/** One hit rect per step interval, plus the last: the card then and `REGION_LEAD` steps on. */
export function regionSteps(from: Box, to: Box): Box[] {
  const at = (i: number) => lerpBox(from, to, Math.min(i, FOLD_STEPS) / FOLD_STEPS);
  return Array.from({ length: FOLD_STEPS + 1 }, (_, i) => bounds(at(i), at(i + REGION_LEAD)));
}

/** `clip-path: inset()` that shows only the part of `el` inside `inner`; negative where `inner` is larger. */
export function insetFor(el: Box, inner: Box): string {
  const px = (v: number) => `${v.toFixed(2)}px`;
  return `inset(${px(inner.y - el.y)} ${px(el.x + el.w - inner.x - inner.w)} ${px(el.y + el.h - inner.y - inner.h)} ${px(inner.x - el.x)})`;
}

type Edges = [number, number, number, number];

function borders(el: HTMLElement): Edges {
  const cs = getComputedStyle(el);
  return [cs.borderTopWidth, cs.borderRightWidth, cs.borderBottomWidth, cs.borderLeftWidth].map((v) => parseFloat(v) || 0) as Edges;
}

export type FoldRun = {
  shell: HTMLElement;
  strip: HTMLElement;
  toPanel: boolean;
  g: FoldGeometry;
  /** Plays the hit region beside the animation; absent in a browser. */
  region?: (rects: Box[], gapMs: number) => void;
  /** The first frame is drawn at once; the card moves only after this settles. */
  hold?: Promise<void>;
};

/** A 2px mark in the bottom-right corner of `win` (its shadow). The backend watches the screen for it
 *  to learn that this layout has really been presented (cover.rs BEACON). */
export function showBeacon(win: Box): HTMLElement {
  const el = document.createElement("div");
  el.className = "fold-beacon";
  el.style.left = `${win.x + win.w - 2}px`;
  el.style.top = `${win.y + win.h - 2}px`;
  document.body.append(el);
  return el;
}

const PLACED = ["left", "top", "height", "clip-path", "opacity"] as const;
let stop: (() => void) | null = null;

/** Puts the page back in flow. Safe to call when no fold is running. */
export function clearFold(shell: HTMLElement, strip: HTMLElement) {
  stop?.();
  for (const el of [shell, strip]) {
    el.classList.remove("fold-measure");
    PLACED.forEach((k) => el.style.removeProperty(k));
  }
  document.querySelectorAll(".ps-fold-frame, .fold-beacon").forEach((el) => el.remove());
  delete document.body.dataset.fold;
}

/** Runs the animation and resolves when it ends; the caller then commits the new shape and calls `clearFold`. */
export function playFold(run: FoldRun): Promise<void> {
  const { shell, strip, toPanel, g } = run;
  const [from, to] = [cardOf(g.from), cardOf(g.to)];
  const [panelCard, stripCard] = toPanel ? [to, from] : [from, to];
  const [incoming, outgoing] = toPanel ? [shell, strip] : [strip, shell];
  const [inCard, outCard] = toPanel ? [panelCard, stripCard] : [stripCard, panelCard];

  // Border widths are read while both still have their in-flow chrome.
  const [bFrom, bTo] = [borders(outgoing), borders(incoming)];
  incoming.classList.remove("fold-measure");
  for (const [el, card] of [[shell, panelCard], [strip, stripCard]] as const) {
    el.style.left = `${card.x}px`;
    el.style.top = `${card.y}px`;
  }
  // In flow the body caps the panel's height; out of flow it needs telling.
  shell.style.height = `${panelCard.h}px`;
  document.body.dataset.fold = "run";
  shell.hidden = false;
  strip.hidden = false;
  const frame = document.createElement("div");
  frame.className = "ps-fold-frame";
  document.body.prepend(frame);

  // One rAF loop writes every moving part, so border, clip and fade can never be a frame apart.
  const draw = (p: number, u: number) => {
    const f = lerpBox(from, to, p);
    const [bt, br, bb, bl] = bFrom.map((w, i) => w + (bTo[i] - w) * p) as Edges;
    frame.style.left = `${f.x}px`;
    frame.style.top = `${f.y}px`;
    frame.style.width = `${f.w}px`;
    frame.style.height = `${f.h}px`;
    frame.style.borderWidth = `${bt}px ${br}px ${bb}px ${bl}px`;
    // Content is clipped to the frame's inside, so it never paints over the travelling border.
    const inner = { x: f.x + bl, y: f.y + bt, w: f.w - bl - br, h: f.h - bt - bb };
    const fade = fadeAt(u);
    outgoing.style.clipPath = insetFor(outCard, inner);
    outgoing.style.opacity = String(fade.out);
    incoming.style.clipPath = insetFor(inCard, inner);
    incoming.style.opacity = String(fade.in);
  };

  draw(0, 0);
  return new Promise((resolve) => {
    let raf = 0;
    let t0 = -1;
    let stopped = false;
    const tick = (now: number) => {
      if (t0 < 0) {
        t0 = now;
        // Started with the first frame, so the region's clock never runs ahead of a shrinking card.
        run.region?.(regionSteps(g.from, g.to), FOLD_MS / FOLD_STEPS);
      }
      const elapsed = now - t0;
      draw(progressAt(elapsed), Math.min(elapsed / FOLD_MS, 1));
      if (elapsed >= FOLD_MS) {
        stop = null;
        return resolve();
      }
      raf = requestAnimationFrame(tick);
    };
    // An interrupting switch stops the loop; the caller has moved on by then.
    stop = () => {
      stopped = true;
      cancelAnimationFrame(raf);
      stop = null;
      resolve();
    };
    void (run.hold ?? Promise.resolve()).then(() => {
      if (!stopped) raf = requestAnimationFrame(tick);
    });
  });
}

/** Resolves once the viewport is `w`×`h` CSS px (the window resize has landed), or after `timeout`. */
export function viewportIs(w: number, h: number, timeout = 250): Promise<void> {
  const ok = () => Math.abs(window.innerWidth - w) <= 1.5 && Math.abs(window.innerHeight - h) <= 1.5;
  if (ok()) return Promise.resolve();
  return new Promise((resolve) => {
    const done = () => {
      window.removeEventListener("resize", check);
      clearTimeout(timer);
      resolve();
    };
    const check = () => ok() && done();
    const timer = setTimeout(done, timeout);
    window.addEventListener("resize", check);
  });
}

/** Resolves after `n` frames have been produced, i.e. what is in the DOM now has been painted. */
export function painted(n = 2): Promise<void> {
  return new Promise((resolve) => {
    const step = () => (n-- > 0 ? requestAnimationFrame(step) : resolve());
    step();
  });
}
