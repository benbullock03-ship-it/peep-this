import { createCards, type Decide } from "./approvalCard";
import { createBar, type Bar } from "./bar";
import { detailFields } from "./details";
import type { Session, TrailEntry } from "./model";
import { requestFocus, toRowView, type IdentityOf, type OnFocus, type RowView } from "./rowView";
import { sortSessions } from "./sort";
import { entryTitle, markOf, shownTarget, trailKey } from "./trail";

export type TrailOf = (sessionId: string) => readonly TrailEntry[] | undefined;

export const EMPTY_TEXT = "No Claude Code sessions running";
const PEEP_SIZE = "32";
const PEEP_SIZE_EXPANDED = "64";
export const FLASH_MS = 1000;
/** Matches the `row-leave` animation in styles.css. */
export const LEAVE_MS = 600;
const TICK_MS = 1000;
/** Matches the `row-enter` animation in styles.css. */
const ENTER_MS = 320;
const STAGGER_MS = 40;
const MAX_STAGGER = 8;

type Details = { el: HTMLElement; values: Map<string, HTMLElement> };

type Row = {
  el: HTMLElement;
  peep: HTMLElement;
  badge: HTMLElement;
  name: HTMLElement;
  task: HTMLElement;
  status: HTMLElement;
  label: HTMLElement;
  ctx: Bar;
  last: Partial<RowView>;
  session: Session;
  details: Details | null;
  trail: { el: HTMLElement; key: string } | null;
  cards?: (pending: Session["pending"]) => number;
};

function el(tag: string, cls: string, parent?: HTMLElement): HTMLElement {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

function createRow(s: Session, decide?: Decide): Row {
  const row = el("div", "row");
  row.dataset.id = s.id;
  row.tabIndex = 0;
  row.setAttribute("aria-expanded", "false");
  const wrap = el("div", "peep", row);
  const peep = document.createElement("peep-bot");
  peep.setAttribute("size", PEEP_SIZE);
  wrap.appendChild(peep);
  const badge = el("span", "badge", wrap);
  const text = el("div", "text", row);
  const name = el("div", "name", text);
  const task = el("div", "task", text);
  const status = el("div", "status", row);
  el("span", "led", status);
  const label = el("span", "label", status);
  let cards: Row["cards"];
  if (decide) {
    const host = el("div", "cards", row);
    host.hidden = true;
    cards = createCards(host, decide);
  }
  const ctx = createBar("ctx");
  row.appendChild(ctx.el);
  return { el: row, peep, badge, name, task, status, label, ctx, last: {}, session: s, details: null, trail: null, cards };
}

// Writes only fields that changed, so <peep-bot> never sees redundant attribute sets.
function updateRow(r: Row, v: RowView) {
  const l = r.last;
  if (l.state !== v.state) {
    r.peep.setAttribute("state", v.state);
    r.el.dataset.state = v.state;
  }
  if (l.color !== v.color) r.peep.setAttribute("color", v.color);
  if (l.accessory !== v.accessory) r.peep.setAttribute("accessory", v.accessory);
  if (l.name !== v.name) {
    r.name.textContent = v.name;
    r.name.title = v.name;
  }
  if (l.task !== v.task) {
    r.task.textContent = v.task;
    r.task.title = v.task;
  }
  if (l.label !== v.label) r.label.textContent = v.label;
  if (l.led !== v.led) r.status.style.setProperty("--led", v.led);
  if (l.subagents !== v.subagents) {
    r.badge.textContent = v.subagents > 0 ? String(v.subagents) : "";
    r.badge.hidden = v.subagents <= 0;
    r.badge.title = v.subagents === 1 ? "1 subagent running" : `${v.subagents} subagents running`;
  }
  if (l.contextPct !== v.contextPct) {
    r.ctx.set(v.contextPct);
    r.ctx.el.title = v.contextPct === null ? "" : `Context ${Math.round(v.contextPct)}% used`;
  }
  if (l.alive !== v.alive) r.el.dataset.alive = String(v.alive);
  r.last = v;
}

function fillDetails(r: Row, now: number) {
  if (!r.details) return;
  for (const f of detailFields(r.session, now)) {
    const v = r.details.values.get(f.key);
    if (!v) continue;
    if (v.textContent !== f.value) v.textContent = f.value;
    const title = f.value === f.full ? "" : f.full;
    if (v.title !== title) v.title = title;
  }
}

function fillTrail(r: Row, entries: readonly TrailEntry[] | undefined) {
  if (!r.trail) return;
  const key = trailKey(entries);
  if (key === r.trail.key) return;
  r.trail.key = key;
  const host = r.trail.el;
  host.replaceChildren();
  host.hidden = !entries?.length;
  const now = Date.now();
  for (const e of entries ?? []) {
    const item = el("div", "trail-item", host);
    item.dataset.outcome = e.outcome;
    item.title = entryTitle(e, now);
    el("span", "trail-mark", item).textContent = markOf(e.outcome);
    el("span", "trail-tool", item).textContent = e.tool;
    el("span", "trail-target", item).textContent = shownTarget(e);
  }
}

function expand(r: Row, trail?: readonly TrailEntry[]) {
  const d = el("dl", "details");
  const values = new Map<string, HTMLElement>();
  for (const f of detailFields(r.session, Date.now())) {
    el("dt", "detail-label", d).textContent = f.label;
    const dd = el("dd", `detail-value detail-${f.key}`, d);
    values.set(f.key, dd);
  }
  r.details = { el: d, values };
  r.el.insertBefore(d, r.ctx.el);
  const t = el("div", "row-trail");
  t.setAttribute("aria-label", "Recent approval decisions");
  r.trail = { el: t, key: "-" };
  r.el.insertBefore(t, r.ctx.el);
  fillTrail(r, trail);
  r.el.classList.add("expanded");
  r.el.setAttribute("aria-expanded", "true");
  r.peep.setAttribute("size", PEEP_SIZE_EXPANDED);
  fillDetails(r, Date.now());
}

function collapse(r: Row) {
  r.details?.el.remove();
  r.details = null;
  r.trail?.el.remove();
  r.trail = null;
  r.el.classList.remove("expanded");
  r.el.setAttribute("aria-expanded", "false");
  r.peep.setAttribute("size", PEEP_SIZE);
}

function enter(row: HTMLElement, delayMs: number) {
  row.style.setProperty("--enter-delay", `${delayMs}ms`);
  row.classList.add("entering");
  // A timer, not animationend: the waiting pulse overrides row-enter, which would then never end.
  setTimeout(() => row.classList.remove("entering"), delayMs + ENTER_MS);
}

function flash(row: HTMLElement) {
  row.classList.remove("flash");
  void row.offsetWidth; // restarts the animation on a repeat click
  row.classList.add("flash");
  clearTimeout(Number(row.dataset.flashTimer));
  row.dataset.flashTimer = String(setTimeout(() => row.classList.remove("flash"), FLASH_MS));
}

export function createList(
  root: HTMLElement,
  identityOf?: IdentityOf,
  onFocus?: OnFocus,
  onDecide?: Decide,
  trailOf?: TrailOf,
) {
  const list = el("div", "list", root);
  const empty = el("div", "empty", root);
  empty.textContent = EMPTY_TEXT;
  const rows = new Map<string, Row>();
  let open: Row | null = null;
  let ticker = 0;
  let firstRender = true;

  function close() {
    if (!open) return;
    collapse(open);
    open = null;
    clearInterval(ticker);
    ticker = 0;
  }

  function toggle(r: Row) {
    const wasOpen = open === r;
    close();
    if (wasOpen) return;
    expand(r, trailOf?.(r.session.id));
    open = r;
    ticker = window.setInterval(() => open && fillDetails(open, Date.now()), TICK_MS);
  }

  const rowOf = (e: Event) => {
    const id = (e.target as Element).closest<HTMLElement>(".row")?.dataset.id;
    return id ? rows.get(id) : undefined;
  };

  if (onFocus) list.classList.add("focusable");
  list.addEventListener("click", (e) => {
    const r = rowOf(e);
    if (!r || r.el.classList.contains("leaving")) return;
    if ((e.target as Element).closest(".cards")) return;
    if (onFocus && (e.target as Element).closest(".peep, .name")) {
      void requestFocus(onFocus, r.session.id, () => flash(r.el));
      return;
    }
    toggle(r);
  });
  list.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    const r = rowOf(e);
    if (!r || e.target !== r.el) return;
    e.preventDefault();
    toggle(r);
  });

  return function render(sessions: readonly Session[]) {
    const sorted = sortSessions(sessions);
    const seen = new Set<string>();
    let prev: Element | null = null;
    for (const s of sorted) {
      seen.add(s.id);
      let r = rows.get(s.id);
      if (!r) {
        r = createRow(s, onDecide);
        rows.set(s.id, r);
        enter(r.el, firstRender ? Math.min(seen.size - 1, MAX_STAGGER) * STAGGER_MS : 0);
      }
      r.session = s;
      if (r.el.classList.contains("leaving")) {
        clearTimeout(Number(r.el.dataset.leaveTimer));
        r.el.classList.remove("leaving");
      }
      updateRow(r, toRowView(s, identityOf));
      r.el.classList.toggle("has-cards", (r.cards?.(s.alive ? s.pending : undefined) ?? 0) > 0);
      if (r === open) {
        fillDetails(r, Date.now());
        fillTrail(r, trailOf?.(s.id));
      }
      // Fading rows hold their slot, so ordering skips over them.
      let next: Element | null = prev ? prev.nextElementSibling : list.firstElementChild;
      while (next && next !== r.el && next.classList.contains("leaving")) next = next.nextElementSibling;
      if (next !== r.el) list.insertBefore(r.el, next);
      prev = r.el;
    }
    for (const [id, r] of rows) {
      if (seen.has(id) || r.el.classList.contains("leaving")) continue;
      if (r === open) close();
      r.el.classList.add("leaving");
      r.el.dataset.leaveTimer = String(
        setTimeout(() => {
          r.el.remove();
          rows.delete(id);
        }, LEAVE_MS),
      );
    }
    empty.hidden = sorted.length > 0;
    if (sorted.length > 0) firstRender = false;
  };
}
