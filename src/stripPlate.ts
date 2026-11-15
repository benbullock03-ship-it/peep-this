import { createHousePeep, HOUSE_SIZE_STRIP, HOUSE_TEXT, houseShown } from "./house";
import type { Session } from "./model";
import type { IdentityOf } from "./rowView";
import { stripView, type StripCell } from "./strip";

const PEEP_SIZE = "40";
/** A press that travels this far (CSS px) is a drag, not a click. */
const DRAG_PX = 4;

export type StripActions = { open: () => void; drag?: () => void };

type Cell = { el: HTMLButtonElement; peep: HTMLElement; fail: HTMLElement; last?: StripCell };

function createCell(): Cell {
  const el = document.createElement("button");
  el.type = "button";
  el.className = "ps-strip-btn";
  const peep = document.createElement("peep-bot");
  peep.setAttribute("size", PEEP_SIZE);
  peep.setAttribute("hover", "off");
  peep.setAttribute("aria-hidden", "true");
  const dot = document.createElement("span");
  dot.className = "ps-dot";
  const fail = document.createElement("span");
  fail.className = "ps-strip-fail";
  fail.hidden = true;
  el.append(peep, dot, fail);
  return { el, peep, fail };
}

// Writes only fields that changed, so <peep-bot> never sees redundant attribute sets.
function updateCell(c: Cell, v: StripCell) {
  const l = c.last;
  if (l?.state !== v.state) {
    c.peep.setAttribute("state", v.state);
    if (l) c.el.classList.remove(`st-${l.state}`);
    c.el.classList.add(`st-${v.state}`);
  }
  if (l?.attention !== v.attention) c.el.classList.toggle("is-att", v.attention);
  if (l?.color !== v.color) c.peep.setAttribute("color", v.color);
  if (l?.accessory !== v.accessory) c.peep.setAttribute("accessory", v.accessory);
  if (l?.name !== v.name || l.label !== v.label || l.shellFailed !== v.shellFailed) {
    const text = `${v.name}: ${v.label}${v.shellFailed ? ", a shell failed" : ""}`;
    c.el.title = text;
    c.el.setAttribute("aria-label", `${text}. Open Peep This`);
  }
  if (l?.shellFailed !== v.shellFailed) c.fail.hidden = !v.shellFailed;
  if (l?.alive !== v.alive) c.el.dataset.alive = String(v.alive);
  c.last = v;
}

/** The whole plate drags the widget; a press that does not move opens the panel. */
function wirePress(host: HTMLElement, actions: StripActions) {
  let down: { x: number; y: number } | null = null;
  let dragged = false;
  host.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    down = { x: e.screenX, y: e.screenY };
    dragged = false;
  });
  // On the window, so a press that leaves the slim plate before it has moved far enough still drags.
  window.addEventListener("pointermove", (e) => {
    if (!down || !(e.buttons & 1)) return;
    if (Math.hypot(e.screenX - down.x, e.screenY - down.y) < DRAG_PX) return;
    down = null;
    if (!actions.drag) return;
    dragged = true;
    actions.drag();
  });
  window.addEventListener("pointerup", () => (down = null));
  host.addEventListener("click", () => {
    if (dragged) {
      dragged = false;
      return;
    }
    actions.open();
  });
}

/** Centre of cell `slot`, or of the `+N` tile past the last cell, along the plate: CSS px from its outer edge (AV-42). */
export function slotCenter(host: HTMLElement, slot: number): number | null {
  const cells = host.querySelectorAll<HTMLElement>(".ps-strip-btn");
  const el = slot < cells.length ? cells[slot] : host.querySelector<HTMLElement>(".ps-strip-more:not([hidden])");
  if (!el) return null;
  const p = host.getBoundingClientRect();
  const r = el.getBoundingClientRect();
  return host.dataset.layout === "row" ? r.left + r.width / 2 - p.left : r.top + r.height / 2 - p.top;
}

export function createStrip(host: HTMLElement, actions: StripActions, identityOf?: IdentityOf) {
  const open = document.createElement("button");
  open.type = "button";
  open.className = "ps-icon";
  open.textContent = "‹";
  open.title = "Open Peep This";
  open.setAttribute("aria-label", "Open Peep This");
  const more = document.createElement("span");
  more.className = "ps-strip-more";
  more.hidden = true;
  const house = document.createElement("span");
  house.className = "ps-strip-house";
  house.title = HOUSE_TEXT;
  house.appendChild(createHousePeep(HOUSE_SIZE_STRIP, false));
  host.append(open, house, more);
  wirePress(host, actions);
  const cells = new Map<string, Cell>();

  return function render(sessions: readonly Session[]) {
    const view = stripView(sessions, identityOf);
    const seen = new Set<string>();
    let prev: Element = open;
    for (const v of view.cells) {
      seen.add(v.id);
      let c = cells.get(v.id);
      if (!c) {
        c = createCell();
        cells.set(v.id, c);
      }
      updateCell(c, v);
      if (prev.nextElementSibling !== c.el) prev.after(c.el);
      prev = c.el;
    }
    for (const [id, c] of cells) {
      if (seen.has(id)) continue;
      c.el.remove();
      cells.delete(id);
    }
    // Strip cells have no fade-out, so the house returns in the same render the last cell leaves.
    house.hidden = !houseShown(sessions);
    more.hidden = view.more === 0;
    more.textContent = `+${view.more}`;
    more.title = `${view.more} more`;
  };
}
