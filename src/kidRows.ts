import { createCards, type Decide } from "./approvalCard";
import { kidView, nextFoldAt, planKids, type KidView } from "./kids";
import type { Agent, PendingApproval, Session } from "./model";

const KID_PEEP_SIZE = "32";

type Kid = {
  el: HTMLElement;
  peep: HTMLElement;
  say: HTMLElement;
  name: HTMLElement;
  desc: HTMLElement;
  label: HTMLElement;
  stats: HTMLElement;
  cards?: (pending: readonly PendingApproval[] | undefined) => number;
  last: Partial<KidView> & { color?: string };
};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

function button(cls: string, parent: HTMLElement): HTMLButtonElement {
  const b = el("button", cls, parent);
  b.type = "button";
  return b;
}

function createKid(id: string, decide?: Decide): Kid {
  const item = el("div", "ps-kid");
  item.dataset.agent = id;
  const row = el("div", "ps-kid-row", item);
  const stand = el("span", "ps-kid-stand", row);
  const peep = document.createElement("peep-bot");
  peep.setAttribute("size", KID_PEEP_SIZE);
  peep.setAttribute("aria-hidden", "true");
  stand.appendChild(peep);
  el("span", "ps-kid-plank", stand);
  const body = el("span", "ps-kid-body", row);
  const say = el("span", "", el("span", "ps-bubble ps-kid-say", body));
  const meta = el("span", "ps-kid-meta", body);
  const name = el("span", "ps-kid-name", meta);
  const desc = el("span", "ps-kid-desc", meta);
  const stats = el("span", "ps-kid-stats", meta);
  const chip = el("span", "ps-chip ps-kid-chip", meta);
  el("span", "ps-dot", chip);
  const label = el("span", "", chip);
  let cards: Kid["cards"];
  if (decide) {
    const host = el("div", "cards ps-kid-cards", item);
    host.hidden = true;
    cards = createCards(host, decide);
  }
  return { el: item, peep, say, name, desc, label, stats, cards, last: {} };
}

function updateKid(k: Kid, v: KidView, color: string) {
  const l = k.last;
  if (l.state !== v.state) {
    k.peep.setAttribute("state", v.state);
    if (l.state) k.el.classList.remove(`st-${l.state}`);
    k.el.classList.add(`st-${v.state}`);
  }
  if (l.color !== color) k.peep.setAttribute("color", color);
  if (l.accessory !== v.accessory) k.peep.setAttribute("accessory", v.accessory);
  if (l.attention !== v.attention) k.el.classList.toggle("is-att", v.attention);
  if (l.finished !== v.finished) k.el.classList.toggle("is-done", v.finished);
  if (l.say !== v.say) {
    k.say.textContent = v.say;
    k.say.parentElement!.title = v.say;
  }
  if (l.name !== v.name) k.name.textContent = v.name;
  if (l.description !== v.description || l.finished !== v.finished) {
    k.desc.textContent = v.description;
    k.desc.title = v.description;
    // Finished rows need the room for duration and tool count; the description stays on the name's title.
    k.desc.hidden = !v.description || v.finished;
    k.name.title = v.description;
  }
  if (l.stats !== v.stats) {
    k.stats.textContent = v.stats;
    k.stats.hidden = !v.stats;
  }
  if (l.label !== v.label) k.label.textContent = v.label;
  if (l.say !== v.say || l.name !== v.name || l.label !== v.label)
    k.el.setAttribute("aria-label", `Subagent ${v.name}: ${v.label}. ${v.say}`);
  k.last = { ...v, color };
}

export type KidsRender = (s: Session, color: string, now: number) => { claimed: Set<string>; foldAt: number | null };

/** Child rows for one session's subagents, diffed by agent id. Returns the request ids shown under a child. */
export function createKids(host: HTMLElement, decide?: Decide): KidsRender {
  host.setAttribute("role", "group");
  host.setAttribute("aria-label", "Subagents");
  const list = el("div", "ps-kid-list", host);
  const more = button("ps-kid-toggle ps-kid-more", host);
  const done = button("ps-kid-toggle ps-kid-done", host);
  const kids = new Map<string, Kid>();
  let allRunning = false;
  let showDone = false;
  let last: { s: Session; color: string } | null = null;

  function draw(s: Session, color: string, now: number) {
    last = { s, color };
    const pending = s.alive ? s.pending : undefined;
    const plan = planKids(s.agents, now, { allRunning, showDone, pending });
    const seen = new Set<string>();
    const claimed = new Set<string>();
    let prev: Element | null = null;
    for (const a of plan.shown) {
      seen.add(a.id);
      let k = kids.get(a.id);
      if (!k) {
        k = createKid(a.id, decide);
        kids.set(a.id, k);
      }
      updateKid(k, kidView(a, pending, now), color);
      const mine = isLive(a) ? (pending ?? []).filter((p) => p.agentId === a.id) : [];
      for (const p of mine) claimed.add(p.id);
      k.cards?.(mine);
      const next: Element | null = prev ? prev.nextElementSibling : list.firstElementChild;
      if (next !== k.el) list.insertBefore(k.el, next);
      prev = k.el;
    }
    for (const [id, k] of kids) {
      if (seen.has(id)) continue;
      k.el.remove();
      kids.delete(id);
    }
    more.hidden = plan.overflow === 0;
    more.textContent = allRunning ? "– show fewer" : `+${plan.more} more`;
    more.setAttribute("aria-expanded", String(allRunning));
    done.hidden = plan.done === 0;
    done.textContent = `✓ done ${plan.done}`;
    done.setAttribute("aria-expanded", String(showDone));
    done.title = showDone ? "Hide finished subagents" : "Show finished subagents";
    host.hidden = plan.shown.length === 0 && plan.overflow === 0 && plan.done === 0;
    return { claimed, foldAt: nextFoldAt(s.agents, now) };
  }

  more.addEventListener("click", () => {
    allRunning = !allRunning;
    if (last) draw(last.s, last.color, Date.now());
  });
  done.addEventListener("click", () => {
    showDone = !showDone;
    if (last) draw(last.s, last.color, Date.now());
  });

  return draw;
}

const isLive = (a: Agent) => a.endedAt === undefined;
