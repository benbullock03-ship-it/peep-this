import { createBar, type Bar } from "./bar";
import type { RateLimits } from "./model";

export type LimitsView = { fiveHour: number; sevenDay: number; at: number };

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
  return { fiveHour, sevenDay, at: rl.at };
}

/** The header shows blocks only (4px blocks leave no room for the %), so the numbers live in the hover title. */
export function limitsTitle(v: LimitsView, at: number, now: number): string {
  return `5h ${v.fiveHour}% · 7d ${v.sevenDay}% · ${ageText(at, now)}`;
}

type Meter = { item: HTMLElement; bar: Bar };

function meter(root: HTMLElement, label: string): Meter {
  const item = document.createElement("span");
  item.className = "ps-meter";
  const bar = createBar("bar--mini");
  item.append(label, bar.el);
  root.appendChild(item);
  return { item, bar };
}

function setMeter(m: Meter, label: string, v: number) {
  m.item.setAttribute("aria-label", `${label} ${v}%`);
  m.bar.set(v);
}

/**
 * Renders the header rate limits into `root`. `refreshAt` lets the hover title use a newer payload time than
 * the last pushed snapshot, since the backend only pushes when the percentages change.
 */
export function createLimits(root: HTMLElement, refreshAt?: () => Promise<number | undefined>) {
  root.hidden = true;
  const five = meter(root, "5h");
  const seven = meter(root, "7d");
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
    if (!v) return;
    last = v;
    setMeter(five, "5h", v.fiveHour);
    setMeter(seven, "7d", v.sevenDay);
    at = Math.max(at, v.at);
    retitle();
  };
}
