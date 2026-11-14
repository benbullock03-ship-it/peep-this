import { findCard } from "./approvalCard";
import { badgeText, inboxOrder, step, successor, type InboxItem } from "./inbox";
import type { Session } from "./model";

export type InboxDeps = {
  header: HTMLElement;
  badge: HTMLElement;
  /** Closes anything covering the list (the settings view). */
  reveal: () => void;
  focusTerminal: (sessionId: string) => void;
  /** Ends the visit; `restore` sends focus back to the window the hotkey was pressed in. */
  leave: (restore: boolean) => void;
};

const FLASH_MS = 480;

/** The keyboard inbox (AV-24): drives the approval cards already mounted under rows. */
export function createInbox(deps: InboxDeps) {
  let order: InboxItem[] = [];
  let active = false;
  let current: string | null = null;
  let hotkey: string | null = null;

  function show(scroll: boolean) {
    const h = current ? findCard(current) : undefined;
    for (const c of document.querySelectorAll<HTMLElement>(".card.is-inbox")) if (c !== h?.el) c.classList.remove("is-inbox");
    if (!h) return;
    h.el.classList.add("is-inbox");
    if (scroll) {
      h.el.closest<HTMLElement>(".ps-kid, .ps-item")?.scrollIntoView({ block: "nearest" });
      h.el.scrollIntoView({ block: "nearest" });
    }
    if (!h.el.contains(document.activeElement)) h.focus();
  }

  function move(id: string | null) {
    if (current && current !== id) findCard(current)?.holdStop();
    current = id;
    show(true);
  }

  function end(restore: boolean) {
    if (!active) return;
    if (current) findCard(current)?.holdStop();
    active = false;
    current = null;
    for (const c of document.querySelectorAll(".card.is-inbox")) c.classList.remove("is-inbox");
    deps.leave(restore);
  }

  function flash() {
    deps.header.classList.remove("is-flash");
    void deps.header.offsetWidth;
    deps.header.classList.add("is-flash");
    clearTimeout(Number(deps.header.dataset.flashTimer));
    deps.header.dataset.flashTimer = String(setTimeout(() => deps.header.classList.remove("is-flash"), FLASH_MS));
  }

  function badge() {
    const n = order.length;
    deps.badge.hidden = n === 0;
    deps.badge.textContent = badgeText(n);
    const what = `${n} pending approval${n === 1 ? "" : "s"}`;
    deps.badge.title = hotkey ? `${what} · ${hotkey} to review` : what;
    deps.badge.setAttribute("aria-label", what);
  }

  function open() {
    if (order.length === 0) {
      flash();
      return;
    }
    deps.reveal();
    active = true;
    move(order[0].id);
  }

  /** Call after every list render, so the cards it looks up are mounted. */
  function sync(sessions: readonly Session[]) {
    const prev = order;
    order = inboxOrder(sessions);
    badge();
    if (!active) return;
    const next = successor(prev, order, current);
    if (next === null) end(true);
    else if (next !== current) move(next);
    else show(false);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (!active || !current) return;
    const h = findCard(current);
    if (e.key === "Escape") {
      e.preventDefault();
      if (!h?.closeReason()) end(true);
      return;
    }
    if ((e.target as Element | null)?.closest?.("input, textarea, select")) return;
    if (e.ctrlKey || e.altKey || e.metaKey) return;
    switch (e.key) {
      case "Tab":
        e.preventDefault();
        move(step(order, current, e.shiftKey ? -1 : 1));
        break;
      case "Enter": {
        e.preventDefault();
        const item = order.find((x) => x.id === current);
        end(false);
        if (item) deps.focusTerminal(item.sessionId);
        break;
      }
      case "y":
      case "Y":
        e.preventDefault();
        if (!e.repeat) h?.allowKey();
        break;
      case "n":
      case "N":
        e.preventDefault();
        if (!e.repeat) h?.deny();
        break;
      case "r":
      case "R":
        e.preventDefault();
        h?.openReason();
        break;
    }
  }

  function onKeyUp(e: KeyboardEvent) {
    if (active && current && (e.key === "y" || e.key === "Y")) findCard(current)?.holdStop();
  }

  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);
  // The user clicked away to another app: forget the way back rather than yank focus later.
  window.addEventListener("blur", () => end(false));

  return {
    open,
    flash,
    sync,
    setHotkey(label: string | null) {
      hotkey = label;
      badge();
    },
    get active() {
      return active;
    },
  };
}
