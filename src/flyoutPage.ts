/** AV-42: the flyout window's page. The backend places the window beside the strip; this lays the rows out in it. */
import "./vendor/peep.js";
import { fillBubble } from "./bubble";
import { layOut, SHADOW, type Side } from "./flyout";
import type { Session, Snapshot } from "./model";
import { MOTION_MS, motionMs } from "./motion";
import { toRowView, type RowView } from "./rowView";
import { initTheme, parsePref } from "./theme";

/** Mirrors `FlyView` in src-tauri/src/flyout.rs. */
type FlyView = { side: Side; origin: number; zone: [number, number]; items: { id: string; at: number }[] };

/** Matches `.ps-fly-tail`: how far the pointer reaches toward the strip, and its width. */
const TAIL = 8;

type Fly = {
  el: HTMLElement;
  item: HTMLElement;
  peep: HTMLElement;
  say: HTMLElement;
  name: HTMLElement;
  label: HTMLElement;
  row: HTMLButtonElement;
  tail: HTMLElement;
  last: Partial<RowView>;
  since: number;
};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

/** The panel's compact row (rows.ts) on its own card. */
function createFly(id: string): Fly {
  const card = el("div", "ps ps-fly");
  const item = el("div", "ps-item", card);
  item.dataset.id = id;
  const row = el("button", "ps-row", item);
  row.type = "button";
  const peep = document.createElement("peep-bot");
  peep.setAttribute("size", "48");
  peep.setAttribute("aria-hidden", "true");
  el("span", "ps-peep", row).appendChild(peep);
  const body = el("span", "ps-body", row);
  const say = el("span", "", el("span", "ps-bubble", body));
  const meta = el("span", "ps-meta", body);
  const name = el("span", "ps-name", meta);
  const chip = el("span", "ps-chip", meta);
  el("span", "ps-dot", chip);
  const label = el("span", "", chip);
  el("span", "ps-plank", item);
  // AV-23's approval card mounts here while the widget is collapsed (spec §12.7).
  el("div", "cards", item).hidden = true;
  const tail = el("span", "ps-fly-tail", card);
  return { el: card, item, peep, say, name, label, row, tail, last: {}, since: 0 };
}

function updateFly(f: Fly, v: RowView) {
  const l = f.last;
  if (l.state !== v.state) {
    f.peep.setAttribute("state", v.state);
    if (l.state) f.item.classList.remove(`st-${l.state}`);
    f.item.classList.add(`st-${v.state}`);
    f.item.dataset.state = v.state;
  }
  if (l.attention !== v.attention) f.item.classList.toggle("is-att", v.attention);
  if (l.color !== v.color) f.peep.setAttribute("color", v.color);
  if (l.accessory !== v.accessory) f.peep.setAttribute("accessory", v.accessory);
  if (l.name !== v.name) {
    f.name.textContent = v.name;
    f.name.title = v.name;
  }
  if (l.label !== v.label) f.label.textContent = v.label;
  if (l.say !== v.say || l.bubble?.tab !== v.bubble.tab) fillBubble(f.say, v.bubble);
  if (l.name !== v.name || l.label !== v.label || l.say !== v.say) f.row.setAttribute("aria-label", `${v.name}: ${v.label}. ${v.say}. Focus its terminal`);
  if (l.alive !== v.alive) f.item.dataset.alive = String(v.alive);
  f.last = v;
}

function flash(item: HTMLElement) {
  item.classList.remove("flash");
  void item.offsetWidth;
  item.classList.add("flash");
  setTimeout(() => item.classList.remove("flash"), 3 * MOTION_MS);
}

/** The card plus its shadow, and the tail, as hit-test rects in zone px. */
function regionOf(side: Side, x: number, y: number, w: number, h: number, tail: number | null): number[][] {
  const rects = [[x, y, w, h]];
  if (tail === null) return rects;
  const [cw, ch] = [w - SHADOW, h - SHADOW];
  const t = tail - TAIL / 2;
  if (side === "left") rects.push([x + cw, y + t, TAIL, TAIL]);
  else if (side === "right") rects.push([x - TAIL, y + t, TAIL, TAIL]);
  else if (side === "down") rects.push([x + t, y - TAIL, TAIL, TAIL]);
  else rects.push([x + t, y + ch, TAIL, TAIL]);
  return rects;
}

async function start() {
  const theme = initTheme();
  window.addEventListener("storage", (e) => {
    if (e.key === "peep.theme") theme.set(parsePref(e.newValue));
  });
  if (!("__TAURI_INTERNALS__" in window)) return;
  const [{ invoke }, { listen, emit }] = await Promise.all([import("@tauri-apps/api/core"), import("@tauri-apps/api/event")]);
  const host = document.querySelector<HTMLElement>("#flyouts")!;
  const flies = new Map<string, Fly>();
  let view: FlyView | null = null;
  let sessions = new Map<string, Session>();
  let frame = 0;
  let lastRegion = "";

  const layout = () => {
    frame = 0;
    if (!view) return;
    const v = view;
    const boxes = v.items
      .filter((i) => flies.has(i.id))
      .map((i) => {
        const f = flies.get(i.id)!;
        return { id: i.id, w: f.el.offsetWidth + SHADOW, h: f.el.offsetHeight + SHADOW, at: v.origin + i.at };
      });
    const placed = layOut(v.side, boxes, { w: v.zone[0], h: v.zone[1] });
    const rects: number[][] = [];
    placed.forEach((p, i) => {
      const f = flies.get(p.id)!;
      f.el.style.left = `${p.x}px`;
      f.el.style.top = `${p.y}px`;
      f.el.dataset.side = v.side;
      f.tail.hidden = p.tail === null;
      if (p.tail !== null) f.el.style.setProperty("--tail", `${p.tail}px`);
      rects.push(...regionOf(v.side, p.x, p.y, boxes[i].w, boxes[i].h, p.tail));
    });
    const key = JSON.stringify(rects);
    if (key === lastRegion) return;
    lastRegion = key;
    // After the new positions are painted, so the region never shows the old ones.
    requestAnimationFrame(() =>
      void invoke("flyout_region", { rects }).catch((err) => console.error("peepthis: flyout_region failed", err)),
    );
  };
  const schedule = () => {
    if (!frame) frame = requestAnimationFrame(layout);
  };
  const ro = new ResizeObserver(schedule);

  const render = () => {
    const want = (view?.items ?? []).filter((i) => sessions.has(i.id));
    const keep = new Set(want.map((i) => i.id));
    for (const [id, f] of flies) {
      if (keep.has(id)) continue;
      ro.unobserve(f.el);
      f.el.remove();
      flies.delete(id);
    }
    const now = Date.now();
    for (const { id } of want) {
      const s = sessions.get(id)!;
      let f = flies.get(id);
      if (!f) {
        f = createFly(id);
        flies.set(id, f);
        if (view) f.el.dataset.side = view.side;
        f.el.classList.add("entering");
        const card = f.el;
        setTimeout(() => card.classList.remove("entering"), motionMs());
        host.appendChild(f.el);
        ro.observe(f.el);
      }
      f.since = s.stateSince;
      updateFly(f, toRowView(s, undefined, now));
    }
    if (!want.length) lastRegion = "";
    schedule();
  };

  host.addEventListener("click", async (e) => {
    const target = e.target as Element;
    if (target.closest(".cards")) return;
    const id = target.closest<HTMLElement>(".ps-item")?.dataset.id;
    const f = id ? flies.get(id) : undefined;
    if (!id || !f) return;
    let ok = false;
    try {
      ok = await invoke<boolean>("focus_session", { sessionId: id });
    } catch (err) {
      console.error("peepthis: focus_session failed", err);
    }
    // The strip's window owns which sessions fly out; it drops this one until it next waits.
    if (ok) void emit("flyout-dismiss", { id, since: f.since });
    else flash(f.item);
  });

  await listen<Snapshot>("sessions", (e) => {
    sessions = new Map(e.payload.sessions.map((s) => [s.id, s]));
    render();
  });
  await listen<FlyView | null>("flyouts", (e) => {
    view = e.payload;
    // The backend clears the region whenever it moves or shows the window.
    lastRegion = "";
    render();
  });
  const [snap, current] = await Promise.all([invoke<Snapshot>("get_snapshot"), invoke<FlyView | null>("get_flyouts")]);
  if (!sessions.size) sessions = new Map(snap.sessions.map((s) => [s.id, s]));
  if (!view && current) {
    view = current;
    lastRegion = "";
  }
  render();
}

start().catch((err) => console.error("peepthis: flyout page failed", err));
