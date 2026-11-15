import type { RuleAnswer, RuleOffer } from "./model";
import { ANSWERS, FILES, FILES_NOTE, offerWhat } from "./rules";

export type AnswerOffer = (offerId: string, answer: RuleAnswer) => Promise<unknown>;

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement, text?: string) {
  const e = document.createElement(tag);
  e.className = cls;
  if (text !== undefined) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

function createOffer(o: RuleOffer, answer: AnswerOffer): HTMLElement {
  const card = el("div", "card card--offer");
  card.dataset.id = o.id;
  card.dataset.tier = o.tier;
  const head = el("div", "card-head", card);
  el("span", "card-tool", head, "Rule");
  el("span", "card-target", head, "3 identical allows");
  const what = offerWhat(o);
  const q = el("p", "offer-q", card);
  q.append("Auto-allow ");
  if (what.tool) q.append(`${what.tool} `);
  el("code", "offer-pattern", q, what.pattern);
  q.append(" in this project?");
  if (o.pattern.includes(FILES)) el("div", "card-note", card, FILES_NOTE);
  el("div", "card-cwd", card, o.cwd).title = o.cwd;
  const actions = el("div", "card-actions", card);
  for (const a of ANSWERS) {
    const b = el("button", `card-btn${a.answer === "yes" ? " card-allow" : ""}`, actions, a.label);
    b.type = "button";
    b.dataset.answer = a.answer;
    b.addEventListener("click", async () => {
      card.classList.add("deciding");
      try {
        await answer(o.id, a.answer);
      } catch (err) {
        console.error("peepthis: answer_rule_offer failed", err);
        card.classList.remove("deciding");
      }
    });
  }
  return card;
}

/** One card per open offer inside `host`, in the order given. */
export function createOffers(host: HTMLElement, answer: AnswerOffer) {
  const cards = new Map<string, HTMLElement>();
  return function render(offers: readonly RuleOffer[]) {
    const live = new Set(offers.map((o) => o.id));
    for (const [id, c] of cards)
      if (!live.has(id)) {
        c.remove();
        cards.delete(id);
      }
    let prev: Element | null = null;
    for (const o of offers) {
      let c = cards.get(o.id);
      if (!c) {
        c = createOffer(o, answer);
        cards.set(o.id, c);
      }
      const next: Element | null = prev ? prev.nextElementSibling : host.firstElementChild;
      if (next !== c) host.insertBefore(c, next);
      prev = c;
    }
    host.hidden = offers.length === 0;
  };
}
