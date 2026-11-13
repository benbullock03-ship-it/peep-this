import type { Session } from "./model";
import { toRowView, type IdentityOf, type RowView } from "./rowView";
import { sortSessions } from "./sort";

export const EMPTY_TEXT = "No Claude Code sessions running";
const PEEP_SIZE = "32";

type Row = {
  el: HTMLElement;
  peep: HTMLElement;
  badge: HTMLElement;
  name: HTMLElement;
  task: HTMLElement;
  status: HTMLElement;
  label: HTMLElement;
  ctx: HTMLElement;
  fill: HTMLElement;
  last: Partial<RowView>;
};

function el(tag: string, cls: string, parent?: HTMLElement): HTMLElement {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

function createRow(id: string): Row {
  const row = el("div", "row");
  row.dataset.id = id;
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
  const ctx = el("div", "ctx", row);
  const fill = el("div", "ctx-fill", ctx);
  return { el: row, peep, badge, name, task, status, label, ctx, fill, last: {} };
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
    r.ctx.hidden = v.contextPct === null;
    r.fill.style.width = v.contextPct === null ? "0" : `${v.contextPct}%`;
    r.ctx.title = v.contextPct === null ? "" : `Context ${Math.round(v.contextPct)}% used`;
  }
  if (l.alive !== v.alive) r.el.dataset.alive = String(v.alive);
  r.last = v;
}

export function createList(root: HTMLElement, identityOf?: IdentityOf) {
  const list = el("div", "list", root);
  const empty = el("div", "empty", root);
  empty.textContent = EMPTY_TEXT;
  const rows = new Map<string, Row>();

  return function render(sessions: readonly Session[]) {
    const sorted = sortSessions(sessions);
    const seen = new Set<string>();
    sorted.forEach((s, i) => {
      seen.add(s.id);
      let r = rows.get(s.id);
      if (!r) {
        r = createRow(s.id);
        rows.set(s.id, r);
      }
      updateRow(r, toRowView(s, identityOf));
      if (list.children[i] !== r.el) list.insertBefore(r.el, list.children[i] ?? null);
    });
    for (const [id, r] of rows) {
      if (!seen.has(id)) {
        r.el.remove();
        rows.delete(id);
      }
    }
    empty.hidden = sorted.length > 0;
  };
}
