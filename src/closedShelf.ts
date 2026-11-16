import { EMOTES } from "./emote";
import { identityFor, type Identity } from "./identity";
import type { ClosedSession } from "./model";
import { isAttention } from "./stateMeta";

/** SPEC §01.6: five rows show, the rest scroll inside. */
export const CLOSED_SHOWN = 5;
const COPIED_MS = 6000;
const TICK_MS = 30_000;

export const resumeCommand = (id: string) => `claude --resume ${id}`;

/** Newest first; a session that came back is a live row again, so it leaves the shelf. */
export function closedOrder(closed: readonly ClosedSession[], liveIds: ReadonlySet<string>): ClosedSession[] {
  return closed.filter((c) => !liveIds.has(c.sessionId)).sort((a, b) => b.endedAt - a.endedAt);
}

/** SPEC §01.3: at most three, in this order. Ending in an attention state is the end chip, not a flag. */
export function closedFlags(c: ClosedSession): string[] {
  const out: string[] = [];
  if (c.flags.uncommitted) out.push("uncommitted");
  if (c.flags.ticketDoing && c.ticket) out.push(`${c.ticket.id} left doing`);
  if (c.flags.noPackup) out.push("no Pack Up");
  return out;
}

export const endLabel = (c: ClosedSession) => (isAttention(c.state) ? `ended ${c.state}` : c.state);

export function agoText(ms: number): string {
  const m = Math.floor(Math.max(0, ms) / 60_000);
  if (m < 1) return "now";
  return m < 60 ? `${m}m` : `${Math.floor(m / 60)}h`;
}

/** "N more below · N to check · show all", dropping zero parts. */
export function moreText(below: number, toCheck: number): string {
  return [below && `${below} more below`, toCheck && `${toCheck} to check`].filter(Boolean).join(" · ");
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

type Row = { el: HTMLButtonElement; peep: HTMLElement; key: string; ago: HTMLElement; flagged: boolean };

/** Copies the command; resolves false when the clipboard refused. */
export type Copy = (text: string) => Promise<boolean>;

const clipboardCopy: Copy = (text) => navigator.clipboard?.writeText(text).then(() => true, () => false) ?? Promise.resolve(false);

/** SPEC §01: the closed shelf under the live rows, inside the same scroll. */
export function createClosedShelf(parent: HTMLElement, copy: Copy = clipboardCopy, identityOf: (cwd: string) => Identity = identityFor) {
  const sec = el("section", "cl", parent);
  sec.setAttribute("aria-label", "Closed sessions");
  const head = el("button", "cl-head", sec);
  head.type = "button";
  el("span", "cl-lbl", head).textContent = "Closed";
  const sub = el("span", "cl-sub", head);
  const check = el("span", "cl-check", head);
  const body = el("div", "cl-body", sec);
  body.id = `cl-body-${Math.random().toString(36).slice(2, 8)}`;
  head.setAttribute("aria-controls", body.id);
  const empty = el("div", "cl-empty", body);
  empty.textContent = "Nothing ended in the last 8 hours.";
  const list = el("div", "cl-list", body);
  const more = el("div", "cl-more", body);
  const moreLine = el("span", "", more);
  const showAll = el("button", "cl-all", more);
  showAll.type = "button";
  showAll.textContent = "show all";
  const hint = el("div", "tray-hint cl-hint", sec);
  hint.setAttribute("role", "status");
  hint.setAttribute("aria-live", "polite");

  const rows = new Map<string, Row>();
  let shown: ClosedSession[] = [];
  let userOpen: boolean | null = null;
  let all = false;
  let copied: { id: string; timer: number } | null = null;
  let hintTimer = 0;
  let last: { closed: readonly ClosedSession[]; live: ReadonlySet<string> } = { closed: [], live: new Set() };

  const syncMore = () => {
    if (all || shown.length <= CLOSED_SHOWN) {
      more.hidden = true;
      return;
    }
    const bottom = list.scrollTop + list.clientHeight;
    let below = 0;
    let toCheck = 0;
    for (const r of rows.values()) {
      if (r.el.offsetTop + r.el.offsetHeight / 2 <= bottom) continue;
      below++;
      if (r.flagged) toCheck++;
    }
    const text = moreText(below, toCheck);
    moreLine.textContent = text && `${text} · `;
    more.hidden = false;
  };
  list.addEventListener("scroll", syncMore);

  const fit = () =>
    requestAnimationFrame(() => {
      const sixth = list.children[CLOSED_SHOWN] as HTMLElement | undefined;
      if (all || !sixth) list.style.maxHeight = "none";
      // Zero while the shelf or the list is hidden; keep the last cap until it can be measured.
      else if (sixth.offsetTop) list.style.maxHeight = `${sixth.offsetTop}px`;
      syncMore();
    });

  const setOpen = () => {
    const open = userOpen ?? shown.length > 0;
    head.setAttribute("aria-expanded", String(open));
    body.hidden = !open;
  };
  head.addEventListener("click", () => {
    userOpen = body.hidden !== false;
    setOpen();
    fit();
  });
  showAll.addEventListener("click", () => {
    all = true;
    fit();
  });

  function build(c: ClosedSession, copiedNow: boolean, id: Identity): Row {
    const flags = closedFlags(c);
    const b = el("button", "cl-row");
    b.type = "button";
    b.dataset.id = c.sessionId;
    b.classList.toggle("is-flagged", flags.length > 0);
    b.classList.toggle("is-copied-row", copiedNow);
    const pcol = el("span", "cl-pcol", b);
    const peep = document.createElement("peep-bot");
    for (const [k, v] of [["state", "idle"], ["color", id.color], ["accessory", id.accessory], ["size", "32"], ["hover", "off"]])
      peep.setAttribute(k, v);
    if (!copiedNow) peep.setAttribute("energy", "1");
    peep.setAttribute("aria-hidden", "true");
    pcol.appendChild(peep);
    el("span", "cl-plank", pcol);
    const main = el("span", "cl-main", b);
    const meta = el("span", "cl-meta", main);
    const name = el("span", "ps-name cl-name", meta);
    name.textContent = c.name;
    name.title = c.name;
    const chip = el("span", `cl-chip st-${c.state}`, meta);
    chip.textContent = endLabel(c);
    const ago = el("span", "cl-ago", meta);
    if (copiedNow) {
      const bub = el("span", "ps-bubble cl-bub", main);
      el("span", "", bub).textContent = "On your clipboard!";
      const e = el("span", "em", bub);
      e.dataset.emote = "wave";
      e.dataset.play = EMOTES.wave.play;
      e.style.setProperty("--row", String(EMOTES.wave.row));
      e.setAttribute("aria-hidden", "true");
    } else {
      const g = el("span", "cl-ghost", main);
      g.textContent = c.bubble;
      g.title = c.bubble;
    }
    if (c.ticket || flags.length) {
      const line = el("span", "cl-flags", main);
      if (c.ticket) el("span", "cl-tk", line).textContent = c.ticket.id;
      for (const f of flags) el("span", "cl-flag", line).textContent = f;
    }
    const flagNote = flags.length ? ` Flags: ${flags.join(", ")}.` : "";
    b.setAttribute("aria-label", `${c.name}, ${endLabel(c)}.${c.ticket ? ` Ticket ${c.ticket.id}.` : ""}${flagNote} Copy ${resumeCommand(c.sessionId)}`);
    return { el: b, peep, ago, flagged: flags.length > 0, key: "" };
  }

  async function onCopy(id: string) {
    const cmd = resumeCommand(id);
    const ok = await copy(cmd);
    clearTimeout(hintTimer);
    hint.replaceChildren();
    if (ok) {
      el("span", "", hint).textContent = `Copied ${cmd} · now paste it in a terminal`;
      el("kbd", "", hint).textContent = "Ctrl+V";
      if (copied) clearTimeout(copied.timer);
      copied = {
        id,
        timer: window.setTimeout(() => {
          copied = null;
          render(last.closed, last.live);
        }, COPIED_MS),
      };
      render(last.closed, last.live);
      rows.get(id)?.el.focus();
    } else {
      el("span", "", hint).textContent = `Couldn't copy. Type ${cmd} in a terminal.`;
    }
    hintTimer = window.setTimeout(() => hint.replaceChildren(), COPIED_MS);
  }
  list.addEventListener("click", (e) => {
    const id = (e.target as Element).closest<HTMLElement>(".cl-row")?.dataset.id;
    if (id) void onCopy(id);
  });

  function render(closed: readonly ClosedSession[], live: ReadonlySet<string>) {
    last = { closed, live };
    shown = closedOrder(closed, live);
    const now = Date.now();
    const toCheck = shown.filter((c) => closedFlags(c).length).length;
    sub.textContent = `last 8h · ${shown.length}`;
    check.textContent = toCheck ? `${toCheck} to check` : "";
    check.hidden = !toCheck;
    empty.hidden = shown.length > 0;
    const seen = new Set<string>();
    let prev: Element | null = null;
    let changed = false;
    for (const c of shown) {
      seen.add(c.sessionId);
      const isCopied = copied?.id === c.sessionId;
      const id = identityOf(c.cwd);
      const key = `${JSON.stringify(c)}|${isCopied}|${id.color}|${id.accessory}`;
      let r = rows.get(c.sessionId);
      if (!r || r.key !== key) {
        const fresh = build(c, isCopied, id);
        fresh.key = key;
        const hadFocus = r?.el === document.activeElement;
        if (r) r.el.replaceWith(fresh.el);
        rows.set(c.sessionId, fresh);
        if (hadFocus) fresh.el.focus();
        r = fresh;
        changed = true;
      }
      const ago = agoText(now - c.endedAt);
      if (r.ago.textContent !== ago) r.ago.textContent = ago;
      const next: Element | null = prev ? prev.nextElementSibling : list.firstElementChild;
      if (next !== r.el) {
        list.insertBefore(r.el, next);
        changed = true;
      }
      prev = r.el;
    }
    for (const [id, r] of rows) {
      if (seen.has(id)) continue;
      r.el.remove();
      rows.delete(id);
      changed = true;
    }
    if (copied && !seen.has(copied.id)) {
      clearTimeout(copied.timer);
      copied = null;
    }
    setOpen();
    if (changed) fit();
  }
  setInterval(() => render(last.closed, last.live), TICK_MS);

  return render;
}
