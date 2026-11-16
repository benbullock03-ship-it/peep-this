import { barLevel, createBar, type Bar } from "./bar";
import type { RateLimits } from "./model";
import { reducedMotion } from "./motion";

export type LimitsView = { fiveHour: number; sevenDay: number; at: number; fiveHourResetsAt?: number; sevenDayResetsAt?: number };

/** Mirrors `limit::WARN_PCT`: from here every peep sweats and the title plate warns. */
export const WARN_PCT = 80;
export const TOAST_PCT = 95;

/** Clamped to 0..100 and rounded; null when not a finite number. */
export function pct(n: number | null | undefined): number | null {
  if (n === undefined || n === null || !Number.isFinite(n)) return null;
  return Math.round(Math.max(0, Math.min(100, n)));
}

export function ageText(at: number, now: number): string {
  const s = Math.max(0, Math.floor((now - at) / 1000));
  if (s < 5) return "updated just now";
  if (s < 60) return `updated ${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `updated ${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `updated ${h}h ago`;
  return `updated ${Math.floor(h / 24)}d ago`;
}

export function toLimitsView(rl: RateLimits | undefined): LimitsView | null {
  if (!rl) return null;
  const fiveHour = pct(rl.fiveHourPct);
  const sevenDay = pct(rl.sevenDayPct);
  if (fiveHour === null || sevenDay === null) return null;
  const v: LimitsView = { fiveHour, sevenDay, at: rl.at };
  if (rl.fiveHourResetsAt !== undefined) v.fiveHourResetsAt = rl.fiveHourResetsAt;
  if (rl.sevenDayResetsAt !== undefined) v.sevenDayResetsAt = rl.sevenDayResetsAt;
  return v;
}

const clock = (at: number) => new Date(at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", hour12: false });

function oneLimit(key: string, p: number, resetsAt: number | undefined, now: number) {
  return resetsAt !== undefined && resetsAt > now ? `${key} ${p}% (resets ${clock(resetsAt)})` : `${key} ${p}%`;
}

/** The hover title: both limits, their reset times when known, and the payload's age. */
export function limitsTitle(v: LimitsView, at: number, now: number): string {
  return `${oneLimit("5h", v.fiveHour, v.fiveHourResetsAt, now)} · ${oneLimit("7d", v.sevenDay, v.sevenDayResetsAt, now)} · ${ageText(at, now)}`;
}

export type Pressure = 0 | 80 | 95 | 100;

/** AV-79 §02: the title reacts to the higher of the two limits. */
export function pressure(v: LimitsView | null): Pressure {
  const p = v ? Math.max(v.fiveHour, v.sevenDay) : 0;
  return p >= 100 ? 100 : p >= TOAST_PCT ? 95 : p >= WARN_PCT ? 80 : 0;
}

export type LimitWarning = { key: "5h" | "7d"; pct: number };

/** The limit a sweating bubble names: the higher one at or over 80%, 5h on a tie. */
export function limitWarning(rl: RateLimits | undefined): LimitWarning | null {
  const v = toLimitsView(rl);
  if (!v || pressure(v) === 0) return null;
  return v.fiveHour >= v.sevenDay ? { key: "5h", pct: v.fiveHour } : { key: "7d", pct: v.sevenDay };
}

type Meter = { item: HTMLElement; bar: Bar; num: HTMLElement };

function meter(root: HTMLElement, label: string): Meter {
  const item = document.createElement("span");
  item.className = "ps-meter";
  const bar = createBar("bar--meter");
  const num = document.createElement("span");
  num.className = "ps-meter-v";
  item.append(label, bar.el, num);
  root.appendChild(item);
  return { item, bar, num };
}

function setMeter(m: Meter, label: string, v: number) {
  m.item.setAttribute("aria-label", `${label} ${v}%`);
  m.item.dataset.level = barLevel(v);
  m.num.textContent = `${v}%`;
  m.bar.set(v);
}

const NUDGE_FRAMES = ["", "sx1", "sx2", ""];
const NUDGE_FRAME_MS = 40;

/** Title pressure on the header around `root`: plate classes, the "· full" tag, and one nudge on crossing 95%. */
function createPressure(root: HTMLElement) {
  const head = root.closest<HTMLElement>(".ps-head");
  const title = head?.querySelector<HTMLElement>(".ps-title");
  const tag = document.createElement("span");
  tag.className = "ps-title-tag";
  tag.textContent = "· full";
  tag.hidden = true;
  title?.insertBefore(tag, title.querySelector(".ps-pending"));
  let last: Pressure = 0;
  let timer = 0;
  const nudge = () => {
    if (!title || reducedMotion()) return;
    clearTimeout(timer);
    let i = 0;
    const step = () => {
      title.classList.remove("sx1", "sx2");
      if (NUDGE_FRAMES[i]) title.classList.add(NUDGE_FRAMES[i]);
      if (++i < NUDGE_FRAMES.length) timer = window.setTimeout(step, NUDGE_FRAME_MS);
    };
    step();
  };
  return (p: Pressure) => {
    if (p === last || !head) return;
    for (const c of ["p80", "p95", "p100"]) head.classList.toggle(c, c === `p${p}`);
    tag.hidden = p !== 100;
    if (p === 95 && last < 95) nudge();
    last = p;
  };
}

/**
 * Renders the header rate limits into `root`. `refreshAt` lets the hover title use a newer payload time than
 * the last pushed snapshot, since the backend only pushes when the percentages change.
 */
export function createLimits(root: HTMLElement, refreshAt?: () => Promise<number | undefined>) {
  root.hidden = true;
  const five = meter(root, "5h");
  const seven = meter(root, "7d");
  const setPressure = createPressure(root);
  let at = 0;
  let last: LimitsView | null = null;
  const retitle = () => {
    if (last) root.title = limitsTitle(last, at, Date.now());
  };

  root.addEventListener("mouseenter", () => {
    if (root.hidden) return;
    retitle();
    refreshAt?.()
      .then((fresh) => {
        if (fresh !== undefined && fresh > at) at = fresh;
        retitle();
      })
      .catch(() => {});
  });

  return function render(rl: RateLimits | undefined) {
    const v = toLimitsView(rl);
    root.hidden = v === null;
    setPressure(pressure(v));
    if (!v) return;
    last = v;
    setMeter(five, "5h", v.fiveHour);
    setMeter(seven, "7d", v.sevenDay);
    at = Math.max(at, v.at);
    retitle();
  };
}
