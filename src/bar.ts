export const BLOCKS = 10;
export const AMBER_FROM = 70;
export const RED_FROM = 90;

export type BarLevel = "neutral" | "amber" | "red";

/** Lit blocks for a used %: nearest block, but any value above 0 lights one. */
export function barBlocks(pct: number): number {
  if (!Number.isFinite(pct) || pct <= 0) return 0;
  const n = Math.round((Math.min(100, pct) / 100) * BLOCKS);
  return Math.max(1, n);
}

export function barLevel(pct: number): BarLevel {
  return pct >= RED_FROM ? "red" : pct >= AMBER_FROM ? "amber" : "neutral";
}

export type Bar = { el: HTMLElement; set(pct: number | null): void };

/** A 10-block segmented bar; hidden while the value is null. */
export function createBar(cls: string): Bar {
  const el = document.createElement("span");
  el.className = `bar ${cls}`;
  el.hidden = true;
  const blocks: HTMLElement[] = [];
  for (let i = 0; i < BLOCKS; i++) {
    const b = document.createElement("span");
    b.className = "bar-block";
    el.appendChild(b);
    blocks.push(b);
  }
  let lit = -1;
  return {
    el,
    set(pct) {
      el.hidden = pct === null;
      if (pct === null) return;
      el.dataset.level = barLevel(pct);
      const n = barBlocks(pct);
      if (n === lit) return;
      lit = n;
      blocks.forEach((b, i) => b.classList.toggle("on", i < n));
    },
  };
}
