/** AV-39: the strip/panel switch, animated inside a window held on the union of both shapes (spec §12.3). */

export type Box = { x: number; y: number; w: number; h: number };
/** Mirrors `FoldView` in src-tauri/src/window.rs: CSS px, window rects with the shadow included. */
export type FoldGeometry = { win: [number, number]; from: Box; to: Box };

/** a: 12 steps over 240ms. b: ease-out glide with a 2-frame snap. c: zoom-rect outline, then the content pops. */
export type FoldVariant = "a" | "b" | "c";
export const FOLD_VARIANTS: readonly FoldVariant[] = ["a", "b", "c"];
export const FOLD_DEFAULT: FoldVariant = "a";

/** Progress at evenly spaced times; `stepped[i]` holds sample i until i+1 is due. */
export type Timeline = { ms: number; samples: number[]; stepped: boolean[] };

/** Matches the `.ps` hard shadow and window.rs SHADOW. */
const SHADOW = 6;
/** The region runs this many samples ahead of a growing card, so IPC lag never clips it. */
export const REGION_LEAD = 3;

const easeOut = (t: number) => 1 - (1 - t) ** 3;

export function timeline(v: FoldVariant): Timeline {
  if (v === "b") {
    const glide = Array.from({ length: 12 }, (_, i) => 0.94 * easeOut(i / 11));
    return { ms: 220, samples: [...glide, 0.97, 1], stepped: [...Array<boolean>(11).fill(false), true, true] };
  }
  const n = v === "c" ? 6 : 12;
  return { ms: v === "c" ? 180 : 240, samples: Array.from({ length: n + 1 }, (_, i) => i / n), stepped: Array<boolean>(n).fill(true) };
}

export function isFoldVariant(v: unknown): v is FoldVariant {
  return FOLD_VARIANTS.includes(v as FoldVariant);
}

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

/** One hit rect per sample: the moving rect now and `REGION_LEAD` samples on, plus anything that stays put. */
export function regionSteps(t: Timeline, from: Box, to: Box, keep: Box[] = []): Box[] {
  const at = (i: number) => lerpBox(from, to, t.samples[Math.min(i, t.samples.length - 1)]);
  return t.samples.map((_, i) => bounds(at(i), at(i + REGION_LEAD), ...keep));
}

/** `clip-path: inset()` that shows only the part of `el` inside `inner`; negative where `inner` is larger. */
export function insetFor(el: Box, inner: Box): string {
  const px = (v: number) => `${v.toFixed(2)}px`;
  return `inset(${px(inner.y - el.y)} ${px(el.x + el.w - inner.x - inner.w)} ${px(el.y + el.h - inner.y - inner.h)} ${px(inner.x - el.x)})`;
}

/** Progress `elapsed` ms in. A stepped interval jumps to its end at once, so the first step lands on the click. */
export function progressAt(t: Timeline, elapsed: number): number {
  const n = t.samples.length - 1;
  if (elapsed >= t.ms) return 1;
  const f = (Math.max(elapsed, 0) / t.ms) * n;
  const i = Math.min(Math.floor(f), n - 1);
  return t.stepped[i] ? t.samples[i + 1] : t.samples[i] + (t.samples[i + 1] - t.samples[i]) * (f - i);
}

/** Cross-fade over time fraction `u`: the outgoing shape is gone by 40%, the incoming one arrives from 20% to 70%. */
export function fadeAt(u: number, stepped: boolean): { out: number; in: number } {
  const clamp = (v: number) => Math.min(1, Math.max(0, v));
  const q = (v: number) => (stepped ? Math.floor(clamp(v) * 4) / 4 : clamp(v));
  return { out: 1 - q(u / 0.4), in: q((u - 0.2) / 0.5) };
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
  variant: FoldVariant;
  /** Plays the hit region beside the animation; absent in a browser. */
  region?: (rects: Box[], gapMs: number) => void;
};

const PLACED = ["left", "top", "height", "clip-path", "opacity"] as const;
let stop: (() => void) | null = null;

/** Puts the page back in flow. Safe to call when no fold is running. */
export function clearFold(shell: HTMLElement, strip: HTMLElement) {
  stop?.();
  for (const el of [shell, strip]) {
    el.classList.remove("fold-measure");
    PLACED.forEach((k) => el.style.removeProperty(k));
  }
  document.querySelectorAll(".ps-fold-frame, .ps-fold-outline").forEach((el) => el.remove());
  delete document.body.dataset.fold;
}

function place(el: HTMLElement, card: Box, fixHeight: boolean) {
  el.style.left = `${card.x}px`;
  el.style.top = `${card.y}px`;
  if (fixHeight) el.style.height = `${card.h}px`;
}

/** Runs the animation and resolves when it ends; the caller then commits the new shape and calls `clearFold`. */
export function playFold(run: FoldRun): Promise<void> {
  const { shell, strip, toPanel, g, variant } = run;
  const t = timeline(variant);
  const [from, to] = [cardOf(g.from), cardOf(g.to)];
  const [panelCard, stripCard] = toPanel ? [to, from] : [from, to];
  const incoming = toPanel ? shell : strip;
  const outgoing = toPanel ? strip : shell;
  // One rAF loop writes every moving part, so border, clip and fade can never be a frame apart.
  let draw: (p: number, u: number) => void;
  let region: Box[];

  // Border widths are read while both still have their in-flow chrome.
  const [bFrom, bTo] = [borders(outgoing), borders(incoming)];
  incoming.classList.remove("fold-measure");
  place(shell, panelCard, true);
  place(strip, stripCard, false);
  const box = (el: HTMLElement, b: Box) => {
    el.style.left = `${b.x}px`;
    el.style.top = `${b.y}px`;
    el.style.width = `${b.w}px`;
    el.style.height = `${b.h}px`;
  };

  if (variant === "c") {
    // Classic zoom rect: the strip stays put, an outline travels, the panel pops in or out.
    document.body.dataset.fold = "zoom";
    shell.hidden = true;
    strip.hidden = false;
    const outline = document.createElement("div");
    outline.className = "ps-fold-outline";
    document.body.append(outline);
    draw = (p) => box(outline, lerpBox(from, to, p));
    region = regionSteps(t, g.from, g.to, [toPanel ? g.from : g.to]);
  } else {
    document.body.dataset.fold = "run";
    shell.hidden = false;
    strip.hidden = false;
    const frame = document.createElement("div");
    frame.className = "ps-fold-frame";
    document.body.prepend(frame);
    const [outCard, inCard] = toPanel ? [stripCard, panelCard] : [panelCard, stripCard];
    draw = (p, u) => {
      const f = lerpBox(from, to, p);
      const [bt, br, bb, bl] = bFrom.map((w, i) => w + (bTo[i] - w) * p) as Edges;
      box(frame, f);
      frame.style.borderWidth = `${bt}px ${br}px ${bb}px ${bl}px`;
      // Content is clipped to the frame's inside, so it never paints over the travelling border.
      const inner = { x: f.x + bl, y: f.y + bt, w: f.w - bl - br, h: f.h - bt - bb };
      const fade = fadeAt(u, t.stepped[0]);
      outgoing.style.clipPath = insetFor(outCard, inner);
      outgoing.style.opacity = String(fade.out);
      incoming.style.clipPath = insetFor(inCard, inner);
      incoming.style.opacity = String(fade.in);
    };
    region = regionSteps(t, g.from, g.to);
  }

  draw(0, 0);
  return new Promise((resolve) => {
    let raf = 0;
    let t0 = -1;
    const tick = (now: number) => {
      if (t0 < 0) {
        t0 = now;
        // Started with the first frame, so the region's clock never runs ahead of a shrinking card.
        run.region?.(region, t.ms / (t.samples.length - 1));
      }
      const elapsed = now - t0;
      draw(progressAt(t, elapsed), Math.min(elapsed / t.ms, 1));
      if (elapsed >= t.ms) {
        stop = null;
        return resolve();
      }
      raf = requestAnimationFrame(tick);
    };
    // An interrupting switch stops the loop; the caller has moved on by then.
    stop = () => {
      cancelAnimationFrame(raf);
      stop = null;
      resolve();
    };
    raf = requestAnimationFrame(tick);
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
