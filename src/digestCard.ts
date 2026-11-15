import { entryAction, factsOf, filesLine, GROUP_LABEL, GROUP_STATE, headline, sections } from "./digest";
import type { Digest, DigestEntry } from "./model";

export type DigestDeps = {
  focus: (sessionId: string) => void;
  openInbox: () => void;
  dismiss: () => void;
};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement, text?: string) {
  const e = document.createElement(tag);
  e.className = cls;
  if (text !== undefined) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

function entryEl(e: DigestEntry, deps: DigestDeps): HTMLElement {
  const action = entryAction(e);
  const item = el(action === "none" ? "div" : "button", "digest-entry", undefined);
  item.dataset.id = e.sessionId;
  if (item instanceof HTMLButtonElement) {
    item.type = "button";
    item.title = action === "inbox" ? "Open the approval inbox" : "Focus the terminal";
    item.addEventListener("click", () => (action === "inbox" ? deps.openInbox() : deps.focus(e.sessionId)));
  }
  const head = el("div", "digest-head", item);
  el("span", "digest-name", head, e.name);
  if (!e.alive) el("span", "digest-ended", head, "ended");
  el("div", "digest-facts", item, factsOf(e).join(" · "));
  const files = filesLine(e);
  if (files) el("div", "digest-files", item, files).title = e.files.join("\n");
  if (e.lastLine) el("div", "digest-line", item, e.lastLine).title = e.lastLine;
  return item;
}

/** AV-32: the while-you-were-away card at the top of the list; `render(null)` removes it.
 *  Esc dismisses it while the widget has focus, unless `escTaken` (the inbox owns Esc while it runs). */
export function createDigestCard(host: HTMLElement, deps: DigestDeps, escTaken: () => boolean) {
  const card = el("section", "card card--digest");
  card.setAttribute("aria-label", "While you were away");
  card.hidden = true;
  host.prepend(card);
  window.addEventListener("keydown", (e) => {
    if (e.key !== "Escape" || card.hidden || e.defaultPrevented || escTaken()) return;
    if ((e.target as Element | null)?.closest?.("input, textarea, select")) return;
    e.preventDefault();
    deps.dismiss();
  });
  return function render(d: Digest | null) {
    card.replaceChildren();
    card.hidden = !d || d.entries.length === 0;
    if (!d || card.hidden) return;
    const top = el("div", "card-head", card);
    el("span", "card-tool", top, "While you were away");
    el("span", "card-target", top, headline(d.since, d.until, d.entries));
    const close = el("button", "digest-close", top, "×");
    close.type = "button";
    close.title = "Dismiss (Esc)";
    close.setAttribute("aria-label", "Dismiss");
    close.addEventListener("click", deps.dismiss);
    for (const s of sections(d.entries)) {
      const sec = el("div", `digest-group st-${GROUP_STATE[s.group]}`, card);
      sec.dataset.group = s.group;
      el("div", "digest-label", sec, `${GROUP_LABEL[s.group]} · ${s.entries.length}`);
      for (const e of s.entries) sec.appendChild(entryEl(e, deps));
    }
  };
}
