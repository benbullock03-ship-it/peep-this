import { FLOPPIES } from "./bubble";
import type { RepoTicket, TicketStatus } from "./model";
import { jumpCommand } from "./tray";

/** FLOPPY-SPEC owner ruling: files keep AV-56's names, chips show the design's. */
export const CHIP: Record<TicketStatus, { label: string; rank: number }> = {
  blocked: { label: "blocked", rank: 0 },
  doing: { label: "active", rank: 1 },
  done: { label: "review", rank: 2 },
  open: { label: "todo", rank: 3 },
  deferred: { label: "parked", rank: 4 },
};

/** FLOPPY-SPEC §03: blocked, active, review, todo, parked; file order within a status. */
export const shelfOrder = (tickets: readonly RepoTicket[]) =>
  tickets.map((t, i) => [t, i] as const).sort(([a, i], [b, j]) => CHIP[a.status].rank - CHIP[b.status].rank || i - j).map(([t]) => t);

const clip = (s?: string) => s?.trim().replace(/\.+$/, "") ?? "";

/** "Parked: <reason>. Wakes when <condition>." for deferred and blocked tickets; "" for the rest. */
export function parkedLine(t: RepoTicket): string {
  if (t.status !== "deferred" && t.status !== "blocked") return "";
  const reason = clip(t.reason);
  const wake = clip(t.wake);
  if (!reason && !wake) return "";
  const head = t.status === "deferred" ? "Parked" : "Blocked";
  return [reason && `${head}: ${reason}.`, wake && `Wakes when ${wake}.`].filter(Boolean).join(" ");
}

// 7 x 7 pixel glyphs drawn at 2x (14 px), FLOPPY-SPEC §04.
const GLYPH: Record<TicketStatus, string[]> = {
  open: ["#######", "#.....#", "#.....#", "#.....#", "#.....#", "#.....#", "#######"],
  doing: ["##.....", "####...", "######.", "#######", "######.", "####...", "##....."],
  done: [".......", ".#####.", "#..#..#", "#.###.#", "#..#..#", ".#####.", "......."],
  blocked: [".#####.", "#######", "#######", "#.....#", "#######", "#######", ".#####."],
  deferred: [".......", ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".##.##.", "......."],
};

const SVG = "http://www.w3.org/2000/svg";

function glyph(status: TicketStatus): SVGSVGElement {
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("viewBox", "0 0 7 7");
  svg.setAttribute("class", "tk-glyph");
  svg.setAttribute("aria-hidden", "true");
  GLYPH[status].forEach((row, y) =>
    [...row].forEach((c, x) => {
      if (c !== "#") return;
      const r = document.createElementNS(SVG, "rect");
      for (const [k, v] of [["x", x], ["y", y], ["width", 1], ["height", 1]]) r.setAttribute(String(k), String(v));
      svg.appendChild(r);
    }),
  );
  return svg;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

export function ticketChip(status: TicketStatus, parent: HTMLElement): HTMLElement {
  const c = el("span", `tk-chip tk-${status}`, parent);
  c.appendChild(glyph(status));
  el("span", "", c).textContent = CHIP[status].label;
  return c;
}

/** Jumps through the tray's hint bar. */
export type ShelfJump = (b: HTMLButtonElement, command: string) => void;

/** The open row's ticket shelf (FLOPPY-SPEC §03). `render(undefined)` hides it: the session has no repo yet. */
export function createShelf(jump: ShelfJump) {
  const sec = el("section", "shelf");
  sec.setAttribute("aria-label", "Tickets");
  const head = el("div", "shelf-head", sec);
  el("span", "ps-k", head).textContent = "Tickets";
  const count = el("span", "shelf-count", head);
  const list = el("div", "shelf-list", sec);
  const more = el("div", "shelf-more", sec);
  let key = "";

  const syncMore = () => {
    const bottom = list.scrollTop + list.clientHeight;
    let n = 0;
    for (const r of list.children) if ((r as HTMLElement).offsetTop + (r as HTMLElement).offsetHeight / 2 > bottom) n++;
    more.textContent = n ? `${n} more below · scroll` : "";
  };
  list.addEventListener("scroll", syncMore);

  function render(tickets: readonly RepoTicket[] | undefined) {
    const k = tickets ? JSON.stringify(tickets) : "";
    if (k === key) return;
    key = k;
    sec.hidden = !tickets;
    list.replaceChildren();
    if (!tickets) return;
    count.textContent = String(tickets.length);
    sec.classList.toggle("is-empty", !tickets.length);
    if (!tickets.length) {
      const empty = el("div", "shelf-empty", list);
      const disk = el("span", "fl fl-32", empty);
      disk.style.setProperty("--col", String(FLOPPIES.ticket.col));
      disk.setAttribute("aria-hidden", "true");
      const text = el("div", "", empty);
      el("strong", "", text).textContent = "No tickets yet.";
      el("span", "", text).textContent = "Pop in the Ticket floppy to write the first one.";
    }
    for (const t of shelfOrder(tickets)) {
      const cmd = jumpCommand("ticket", t.id);
      const b = el("button", `shelf-row is-${t.status}`, list);
      b.type = "button";
      b.dataset.id = t.id;
      el("span", "shelf-id", b).textContent = t.id;
      el("span", "shelf-title", b).textContent = t.title;
      el("span", "shelf-jump", b).textContent = "Jump";
      ticketChip(t.status, b);
      const note = parkedLine(t);
      if (note) el("span", "shelf-note", b).textContent = note;
      b.title = t.title;
      b.setAttribute("aria-label", `${t.id} ${t.title}, ${CHIP[t.status].label}. ${note} Jump: copy ${cmd}`.replace(/\s+/g, " "));
      b.addEventListener("click", () => jump(b, cmd));
    }
    more.textContent = "";
    requestAnimationFrame(() => {
      // Five rows, whatever their height (parked rows carry a second line).
      const sixth = list.children[5] as HTMLElement | undefined;
      if (!sixth) list.style.maxHeight = "none";
      else if (sixth.offsetTop) list.style.maxHeight = `${sixth.offsetTop}px`;
      syncMore();
    });
  }

  return { el: sec, render };
}
