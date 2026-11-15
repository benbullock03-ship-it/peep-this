import type { Session, Shell } from "./model";
import { charsThatFit, isEnded, middleTruncate, nextShellFoldAt, planShells, shellLabel, shellRuntime, shellTitle } from "./shells";

/** Matches `.ps-shell-cmd` font-size. */
const CMD_FONT_PX = 11;
const TICK_MS = 1000;

type Chip = {
  el: HTMLElement;
  cmd: HTMLElement;
  time: HTMLElement;
  label: HTMLElement;
  shell: Shell;
  fit: number;
  last: { state?: string; label?: string; title?: string; command?: string };
};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

function fitCommand(c: Chip) {
  const width = c.cmd.clientWidth;
  // Hidden (strip mode, a fold-measure pass): keep the full text until the observer sees a real width.
  const max = width > 0 ? charsThatFit(width, CMD_FONT_PX) : Infinity;
  const text = Number.isFinite(max) ? middleTruncate(c.shell.command, max) : c.shell.command;
  if (c.cmd.textContent !== text) c.cmd.textContent = text;
  c.fit = width;
}

export type ShellsRender = (s: Session, now: number) => { foldAt: number | null };

/** Chip rows for one session's background shells, diffed by shell id. */
export function createShells(host: HTMLElement): ShellsRender {
  host.setAttribute("role", "group");
  host.setAttribute("aria-label", "Background shells");
  const list = el("div", "ps-shell-list", host);
  const ended = el("button", "ps-kid-toggle ps-shell-ended", host);
  ended.type = "button";
  const chips = new Map<string, Chip>();
  const resize = new ResizeObserver((entries) => {
    for (const e of entries) {
      const c = chips.get((e.target as HTMLElement).closest<HTMLElement>(".ps-shell")?.dataset.shell ?? "");
      if (c && c.cmd.clientWidth !== c.fit) fitCommand(c);
    }
  });
  let showEnded = false;
  let last: Session | null = null;
  let ticker = 0;

  function tick() {
    if (!host.isConnected) {
      clearInterval(ticker);
      ticker = 0;
      return;
    }
    const now = Date.now();
    for (const c of chips.values()) if (!isEnded(c.shell)) c.time.textContent = shellRuntime(c.shell, now);
  }

  function create(sh: Shell): Chip {
    const item = el("div", "ps-shell");
    item.dataset.shell = sh.id;
    el("span", "ps-shell-glyph", item).textContent = ">_";
    const cmd = el("span", "ps-shell-cmd", item);
    const time = el("span", "ps-shell-time", item);
    const label = el("span", "ps-shell-state", item);
    const c: Chip = { el: item, cmd, time, label, shell: sh, fit: -1, last: {} };
    resize.observe(cmd);
    return c;
  }

  function update(c: Chip, sh: Shell, now: number) {
    c.shell = sh;
    const l = c.last;
    if (l.state !== sh.state) {
      if (l.state) c.el.classList.remove(`sh-${l.state}`);
      c.el.classList.add(`sh-${sh.state}`);
      c.el.classList.toggle("is-ended", isEnded(sh));
    }
    if (l.command !== sh.command) fitCommand(c);
    c.time.textContent = shellRuntime(sh, now);
    const label = shellLabel(sh);
    if (l.label !== label) c.label.textContent = label;
    const title = shellTitle(sh);
    if (l.title !== title) c.el.title = title;
    if (l.label !== label || l.command !== sh.command) c.el.setAttribute("aria-label", `Shell ${sh.command}: ${label}`);
    c.last = { state: sh.state, label, title, command: sh.command };
  }

  function draw(s: Session, now: number) {
    last = s;
    const plan = planShells(s.shells, now, showEnded);
    const seen = new Set<string>();
    let prev: Element | null = null;
    for (const sh of plan.shown) {
      seen.add(sh.id);
      let c = chips.get(sh.id);
      if (!c) {
        c = create(sh);
        chips.set(sh.id, c);
      }
      const next: Element | null = prev ? prev.nextElementSibling : list.firstElementChild;
      if (next !== c.el) list.insertBefore(c.el, next);
      update(c, sh, now);
      prev = c.el;
    }
    for (const [id, c] of chips) {
      if (seen.has(id)) continue;
      resize.unobserve(c.cmd);
      c.el.remove();
      chips.delete(id);
    }
    ended.hidden = plan.ended === 0;
    ended.textContent = `>_ ended ${plan.ended}`;
    ended.setAttribute("aria-expanded", String(showEnded));
    ended.title = showEnded ? "Hide ended shells" : "Show ended shells";
    host.hidden = plan.shown.length === 0 && plan.ended === 0;
    const live = plan.shown.some((sh) => !isEnded(sh));
    if (live && !ticker) ticker = window.setInterval(tick, TICK_MS);
    if (!live && ticker) {
      clearInterval(ticker);
      ticker = 0;
    }
    return { foldAt: nextShellFoldAt(s.shells, now) };
  }

  ended.addEventListener("click", () => {
    showEnded = !showEnded;
    if (last) draw(last, Date.now());
  });

  return draw;
}
