import {
  byOldest,
  createHold,
  diffFor,
  formatArgs,
  HOLD_MS,
  pushReason,
  riskParts,
  SHELL_TOOLS,
  targetOf,
} from "./approval";
import type { PendingApproval } from "./model";

export type Decide = (requestId: string, decision: "allow" | "deny", reason?: string) => Promise<boolean>;

const REASONS_KEY = "peepthis.denyReasons";

function loadReasons(): string[] {
  try {
    const v = JSON.parse(localStorage.getItem(REASONS_KEY) ?? "[]");
    return Array.isArray(v) ? v.filter((x) => typeof x === "string") : [];
  } catch {
    return [];
  }
}

function saveReason(reason: string) {
  try {
    localStorage.setItem(REASONS_KEY, JSON.stringify(pushReason(loadReasons(), reason)));
  } catch {
    // storage blocked: the reasons list is a convenience only
  }
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement, text?: string) {
  const e = document.createElement(tag);
  e.className = cls;
  if (text !== undefined) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

function button(cls: string, label: string, parent: HTMLElement) {
  const b = el("button", `card-btn ${cls}`, parent, label);
  b.type = "button";
  return b;
}

function marked(parent: HTMLElement, text: string, parts: ReturnType<typeof riskParts>) {
  if (!parts) {
    parent.textContent = text;
    return;
  }
  parent.append(parts.before);
  el("mark", "card-risk", parent, parts.risky);
  parent.append(parts.after);
}

function body(card: HTMLElement, p: PendingApproval) {
  const diff = diffFor(p.tool, p.input);
  if (SHELL_TOOLS.has(p.tool)) {
    const cmd = el("pre", "card-cmd", card);
    marked(cmd, String(p.input.command ?? ""), riskParts(p, "command"));
    if (typeof p.input.description === "string" && p.input.description)
      el("div", "card-note", card, p.input.description);
    el("div", "card-cwd", card, p.cwd).title = p.cwd;
  } else if (diff) {
    const box = el("div", "card-diff", card);
    for (const l of diff.lines) el("div", `diff-line diff-${l.kind}`, box, l.text || " ");
    if (diff.hidden) el("div", "diff-line diff-more", box, `… ${diff.hidden} more lines`);
    if (p.input.replace_all === true) el("div", "card-note", card, "Replaces every occurrence");
  } else {
    const dl = el("dl", "card-args", card);
    for (const a of formatArgs(p.input)) {
      el("dt", "", dl, a.key);
      el("dd", "", dl, a.value).title = a.value;
    }
  }
}

type Card = { el: HTMLElement; dispose: () => void };

function createCard(p: PendingApproval, decide: Decide): Card {
  const tier = p.risk.tier;
  const card = el("div", "card");
  card.dataset.tier = tier;
  card.dataset.id = p.id;

  const head = el("div", "card-head", card);
  el("span", "card-tool", head, p.tool);
  const target = el("span", "card-target", head);
  const path = targetOf(p);
  const pathField = p.input.file_path !== undefined ? "file_path" : "notebook_path";
  const pathParts = riskParts(p, pathField);
  if (pathParts && pathParts.before + pathParts.risky + pathParts.after !== path) target.textContent = path;
  else marked(target, path, pathParts);
  target.title = path;
  if (p.agentId) el("span", "card-agent", head, "subagent");
  const chip = el("span", "card-tier", head, tier);
  chip.title = p.risk.reason;

  body(card, p);

  const total = Math.max(1, p.releaseAt - p.receivedAt);
  const elapsed = Math.min(total, Math.max(0, Date.now() - p.receivedAt));
  const countdown = el("div", "card-countdown", card);
  const drain = el("div", "card-countdown-fill", countdown);
  drain.style.animationDuration = `${total}ms`;
  drain.style.animationDelay = `-${elapsed}ms`;
  countdown.title = "The terminal prompt takes over when this runs out";

  const actions = el("div", "card-actions", card);
  const allow = button("card-allow", tier === "destructive" ? "Hold to allow" : "Allow", actions);
  const deny = button("card-deny", "Deny", actions);
  const withReason = button("card-reason-toggle", "Deny with reason", actions);

  const reasonBox = el("div", "card-reason", card);
  reasonBox.hidden = true;
  const input = el("input", "card-reason-input", reasonBox);
  input.type = "text";
  input.placeholder = "Tell Claude why, then Enter";
  input.maxLength = 500;
  const recent = el("div", "card-reasons", reasonBox);

  let busy = false;
  const send = async (decision: "allow" | "deny", reason?: string) => {
    if (busy) return;
    busy = true;
    card.classList.add("deciding");
    try {
      await decide(p.id, decision, reason);
      if (reason) saveReason(reason);
    } catch (err) {
      console.error("peepthis: decide_approval failed", err);
      busy = false;
      card.classList.remove("deciding");
    }
  };

  if (tier === "destructive") {
    allow.style.setProperty("--hold-ms", `${HOLD_MS}ms`);
    const hold = createHold(() => {
      allow.classList.remove("holding");
      void send("allow");
    });
    const start = () => {
      if (busy) return;
      allow.classList.add("holding");
      hold.start();
    };
    const stop = () => {
      allow.classList.remove("holding");
      hold.cancel();
    };
    allow.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      allow.setPointerCapture(e.pointerId);
      start();
    });
    allow.addEventListener("pointerup", stop);
    allow.addEventListener("pointercancel", stop);
    allow.addEventListener("lostpointercapture", stop);
    allow.addEventListener("keydown", (e) => {
      if ((e.key === " " || e.key === "Enter") && !e.repeat) start();
      if (e.key === " " || e.key === "Enter") e.preventDefault();
    });
    allow.addEventListener("keyup", stop);
    allow.addEventListener("blur", stop);
  } else {
    allow.addEventListener("click", () => void send("allow"));
  }
  deny.addEventListener("click", () => void send("deny"));

  withReason.addEventListener("click", () => {
    reasonBox.hidden = !reasonBox.hidden;
    if (reasonBox.hidden) return;
    recent.replaceChildren();
    for (const r of loadReasons()) {
      const b = el("button", "card-reason-chip", recent, r);
      b.type = "button";
      b.title = r;
      b.addEventListener("click", () => {
        input.value = r;
        input.focus();
      });
    }
    input.focus();
  });
  input.addEventListener("keydown", (e) => {
    if (e.key !== "Enter") return;
    e.preventDefault();
    const reason = input.value.trim();
    if (reason) void send("deny", reason);
  });

  return { el: card, dispose: () => card.remove() };
}

/** Keeps one card per pending request inside `host`, oldest first, so typed reasons survive re-renders. */
export function createCards(host: HTMLElement, decide: Decide) {
  const cards = new Map<string, Card>();
  return function render(pending: readonly PendingApproval[] | undefined) {
    const sorted = (pending ?? []).slice().sort(byOldest);
    const live = new Set(sorted.map((p) => p.id));
    for (const [id, c] of cards)
      if (!live.has(id)) {
        c.dispose();
        cards.delete(id);
      }
    let prev: Element | null = null;
    for (const p of sorted) {
      let c = cards.get(p.id);
      if (!c) {
        c = createCard(p, decide);
        cards.set(p.id, c);
      }
      const next: Element | null = prev ? prev.nextElementSibling : host.firstElementChild;
      if (next !== c.el) host.insertBefore(c.el, next);
      prev = c.el;
    }
    host.hidden = sorted.length === 0;
    return sorted.length;
  };
}
