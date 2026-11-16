import { createCards, type Decide } from "./approvalCard";
import { fillBubble, FLOPPIES } from "./bubble";
import { bindHold } from "./hold";
import { createBar, type Bar } from "./bar";
import { createArc } from "./arc";
import { openFacts } from "./details";
import { clashLinks, clashTags, EMOTES, type Emote } from "./emote";
import { createHousePeep, HOUSE_SIZE_PANEL, HOUSE_TEXT, houseShown } from "./house";
import { createKids, type KidsRender } from "./kidRows";
import { limitWarning, type LimitWarning } from "./limits";
import type { RateLimits, Repo, RepoTicket, RuleOffer, Session, Tip, TrailEntry } from "./model";
import { MOTION_MS, motionMs, reducedMotion } from "./motion";
import type { LookStore } from "./look";
import { createPicker } from "./picker";
import { requestFocus, toRowView, type IdentityOf, type OnFocus, type RowView } from "./rowView";
import { createOffers, type AnswerOffer } from "./ruleOffer";
import { createShelf } from "./shelf";
import { createShells, type ShellsRender } from "./shellRows";
import { sortSessions } from "./sort";
import { entryTitle, markOf, shownTarget, trailKey } from "./trail";
import { createTray, jumpCommand, type Jump } from "./tray";

export type TrailOf = (sessionId: string) => readonly TrailEntry[] | undefined;
/** `end_session` (AV-96): rejects with the reason when the session was not ended. */
export type EndSession = (sessionId: string) => Promise<unknown>;

/** AV-31: offers under the row, and revoking a rule from its trail entry. */
export type RowRules = {
  offersOf: (sessionId: string) => readonly RuleOffer[];
  answer: AnswerOffer;
  revoke: (ruleId: string) => Promise<unknown>;
  live: (ruleId: string) => boolean;
};

const PEEP_SIZE = "48";
const PEEP_SIZE_OPEN = "96";
/** `row-flash` runs three motion units; the timer also ends the static reduced-motion flash. */
export const FLASH_MS = 3 * MOTION_MS;
/** `floppy-eject` / `floppy-insert` in styles.css: 80ms frames (FLOPPY-SPEC §01). */
const EJECT_MS = 400;
const INSERT_MS = 320;
const TICK_MS = 1000;
/** The tray's "Copied" and hint timings (FLOPPY-SPEC §08). */
const COPIED_MS = 1200;
const HINT_MS = 6000;
const END_LABEL = "Hold to end session";
const STAGGER_MS = 40;
const MAX_STAGGER = 8;

type Facts = {
  el: HTMLElement;
  since: HTMLElement;
  model: HTMLElement;
  kind: HTMLElement;
  branch: HTMLElement;
  tool: HTMLElement;
  ctx: Bar;
  ctxValue: HTMLElement;
  cwd: HTMLElement;
  trail: HTMLElement;
  trailKey: string;
  clearHint: () => void;
  shelf: (tickets: readonly RepoTicket[] | undefined) => void;
  arc: (repo: Repo | undefined) => void;
  locks: (repo: Repo | undefined) => void;
  end?: HTMLButtonElement;
  pick?: (s: Session) => void;
};

type Row = {
  el: HTMLElement;
  closed: HTMLButtonElement;
  head: HTMLButtonElement;
  stage: HTMLElement;
  peepWrap: HTMLElement;
  peep: HTMLElement;
  // [closed, open] copies of the same text.
  names: HTMLElement[];
  labels: HTMLElement[];
  says: HTMLElement[];
  plank: HTMLElement;
  facts: Facts;
  last: Partial<RowView>;
  session: Session;
  cards?: (pending: Session["pending"]) => number;
  offers?: (offers: readonly RuleOffer[]) => void;
  kids: KidsRender;
  shells: ShellsRender;
  /** Red dot on the peep for a failed shell; survives the chip folding and the row closing. */
  fail: HTMLElement;
  floppy?: HTMLElement;
  repo?: Repo;
  /** FLOPPY-SPEC §09 buttons; under the plank, since the closed row is itself a button. */
  nudge: HTMLElement;
  nudgePack: HTMLButtonElement;
  nudgeLater: HTMLButtonElement;
  tipOk: HTMLButtonElement;
  /** A failed end, shown in the bubble over the snapshot's text until it times out. */
  endNote?: string;
  endTimer?: number;
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

function chip(parent: HTMLElement): HTMLElement {
  const c = el("span", "ps-chip", parent);
  el("span", "ps-dot", c);
  return el("span", "", c);
}

function bubble(parent: HTMLElement): HTMLElement {
  return el("span", "", el("span", "ps-bubble", parent));
}

function createFacts(jump: Jump, withEnd: boolean, looks?: LookStore): Facts {
  const box = el("div", "ps-facts");
  const tray = createTray(jump);
  box.appendChild(tray.el);
  const arc = createArc((b, cmd) => void tray.fire(b, cmd));
  box.appendChild(arc.el);
  const shelf = createShelf((b, cmd) => void tray.fire(b, cmd));
  box.appendChild(shelf.el);
  const tags = el("div", "ps-tags", box);
  const since = el("span", "ps-tag is-st", tags);
  const model = el("span", "ps-tag", tags);
  const kind = el("span", "ps-tag", tags);
  const branch = el("span", "ps-tag is-branch", tags);
  const toolRow = el("div", "ps-fact", box);
  el("span", "ps-k", toolRow).textContent = "Last tool";
  const tool = el("span", "ps-v ps-v--clamp", toolRow);
  const ctxRow = el("div", "ps-fact", box);
  el("span", "ps-k", ctxRow).textContent = "Context";
  const ctx = createBar("bar--fact");
  ctxRow.appendChild(ctx.el);
  const ctxValue = el("span", "", ctxRow);
  const cwd = el("div", "ps-path", box);
  const picker = looks ? createPicker(looks) : null;
  if (picker) box.appendChild(picker.el);
  const trail = el("div", "ps-trail", box);
  trail.setAttribute("aria-label", "Recent approval decisions");
  let end: HTMLButtonElement | undefined;
  if (withEnd) {
    end = button("ps-end", el("div", "ps-end-row", box));
    end.textContent = END_LABEL;
    end.title = "Hold for 1 second to end this session's process";
  }
  return { el: box, since, model, kind, branch, tool, ctx, ctxValue, cwd, trail, trailKey: "-", clearHint: tray.clear, shelf: shelf.render, arc: arc.render, locks: tray.lock, end, pick: picker?.fill };
}

function createRow(s: Session, jump: Jump, decide?: Decide, rules?: RowRules, withEnd = false, looks?: LookStore): Row {
  const item = el("div", "ps-item");
  item.dataset.id = s.id;

  const closed = button("ps-row", item);
  closed.setAttribute("aria-expanded", "false");
  const peepWrap = el("span", "ps-peep", closed);
  const peep = document.createElement("peep-bot");
  peep.setAttribute("size", PEEP_SIZE);
  peep.setAttribute("aria-hidden", "true");
  peepWrap.appendChild(peep);
  const body = el("span", "ps-body", closed);
  const say = bubble(body);
  const meta = el("span", "ps-meta", body);
  const name = el("span", "ps-name", meta);
  const label = chip(meta);

  const head = button("ps-openhead", item);
  head.setAttribute("aria-expanded", "true");
  head.hidden = true;
  const openName = el("span", "ps-name", head);
  const openLabel = chip(head);
  const stage = el("div", "ps-stage", item);
  stage.hidden = true;
  const openSay = bubble(stage);

  const plank = el("span", "ps-plank", item);
  const nudge = el("div", "ps-nudge", item);
  nudge.hidden = true;
  const pack = button("nudge-pack", nudge);
  const disk = el("span", "fl", pack);
  disk.style.setProperty("--col", String(FLOPPIES.packup.col));
  disk.setAttribute("aria-hidden", "true");
  el("span", "nudge-pack-label", pack).textContent = "Pack up";
  pack.setAttribute("aria-label", `Pack up: copy ${jumpCommand("packup")}`);
  const nudgeLater = button("nudge-later", nudge);
  nudgeLater.textContent = "Not yet";
  const tipOk = button("nudge-later tip-ok", nudge);
  tipOk.textContent = "Got it";
  const facts = createFacts(jump, withEnd, looks);

  let cards: Row["cards"];
  if (decide) {
    const host = el("div", "cards", item);
    host.hidden = true;
    cards = createCards(host, decide);
  }
  let offers: Row["offers"];
  if (rules) {
    const host = el("div", "cards", item);
    host.hidden = true;
    offers = createOffers(host, rules.answer);
  }
  const kidsHost = el("div", "ps-kids", item);
  kidsHost.hidden = true;
  const shellsHost = el("div", "ps-kids ps-shells", item);
  shellsHost.hidden = true;
  const fail = el("span", "ps-fail", peepWrap);
  fail.hidden = true;
  return {
    el: item,
    closed,
    head,
    stage,
    peepWrap,
    peep,
    names: [name, openName],
    labels: [label, openLabel],
    says: [say, openSay],
    plank,
    facts,
    last: {},
    session: s,
    cards,
    offers,
    kids: createKids(kidsHost, decide),
    shells: createShells(shellsHost),
    fail,
    nudge,
    nudgePack: pack,
    nudgeLater,
    tipOk,
  };
}

/** Re-created on each change so the stepped frames start over. */
function setEmote(r: Row, emote: Emote | null) {
  for (const say of r.says) {
    const host = say.parentElement!;
    host.querySelector(":scope > .em")?.remove();
    if (!emote) continue;
    const e = el("span", "em", host);
    e.dataset.emote = emote;
    e.dataset.play = EMOTES[emote].play;
    e.style.setProperty("--row", String(EMOTES[emote].row));
    e.setAttribute("aria-hidden", "true");
  }
}

// Writes only fields that changed, so <peep-bot> never sees redundant attribute sets.
function updateRow(r: Row, v: RowView) {
  const l = r.last;
  if (l.state !== v.state) {
    if (l.state) r.facts.clearHint();
    r.peep.setAttribute("state", v.state);
    if (l.state) r.el.classList.remove(`st-${l.state}`);
    r.el.classList.add(`st-${v.state}`);
    r.el.dataset.state = v.state;
  }
  if (l.attention !== v.attention) r.el.classList.toggle("is-att", v.attention);
  if (l.color !== v.color) r.peep.setAttribute("color", v.color);
  if (l.accessory !== v.accessory) r.peep.setAttribute("accessory", v.accessory);
  if (l.name !== v.name) {
    for (const n of r.names) {
      n.textContent = v.name;
      n.title = v.name;
    }
  }
  if (l.label !== v.label) for (const x of r.labels) x.textContent = v.label;
  if (!r.endNote && (l.say !== v.say || l.bubble?.tab !== v.bubble.tab || l.bubble?.bolt !== v.bubble.bolt))
    for (const x of r.says) fillBubble(x, v.bubble);
  if (l.emote !== v.emote) setEmote(r, v.emote);
  if (l.nudge !== v.nudge || l.tip !== v.tip) {
    r.el.classList.toggle("is-nudge", !!v.nudge || !!v.tip);
    r.nudge.hidden = !v.nudge && !v.tip;
    r.nudgePack.hidden = !v.nudge && v.tip !== "crowded";
    r.nudgeLater.hidden = v.nudge !== "first";
    r.tipOk.hidden = !v.tip;
  }
  if (l.floppy !== v.floppy) seat(r, v.floppy);
  if (l.shellNote !== v.shellNote) {
    r.fail.hidden = !v.shellNote;
    r.fail.title = v.shellNote;
  }
  if (l.name !== v.name || l.label !== v.label || l.say !== v.say || l.shellNote !== v.shellNote)
    r.closed.setAttribute("aria-label", `${v.name}: ${v.label}${v.shellNote ? ", a shell failed" : ""}. ${v.say}`);
  if (l.alive !== v.alive) r.el.dataset.alive = String(v.alive);
  r.last = v;
}

/** FLOPPY-SPEC §01: the disk sits in the plank slot while a skill runs, and plays the eject frames when it clears. */
function seat(r: Row, col: number | null) {
  const f = r.floppy;
  if (col === null) {
    if (!f) return;
    r.floppy = undefined;
    f.classList.add("is-out");
    setTimeout(() => f.remove(), reducedMotion() ? 0 : EJECT_MS);
    return;
  }
  if (!f) {
    const e = (r.floppy = el("span", "ps-floppy is-in", r.plank));
    e.setAttribute("aria-hidden", "true");
    // Re-sorting moves the row, which would replay the insert frames.
    setTimeout(() => e.classList.remove("is-in"), INSERT_MS);
  }
  r.floppy!.style.setProperty("--col", String(col));
}

function setText(e: HTMLElement, text: string) {
  if (e.textContent !== text) e.textContent = text;
}

function fillFacts(r: Row, now: number) {
  const f = openFacts(r.session, now);
  const x = r.facts;
  setText(x.since, f.since);
  setText(x.model, f.model);
  x.model.hidden = !f.model;
  setText(x.kind, f.kind);
  x.kind.hidden = !f.kind;
  setText(x.branch, f.branch);
  x.branch.hidden = !f.branch;
  x.branch.title = f.branch ? `Git branch ${f.branch}` : "";
  setText(x.tool, f.tool);
  x.tool.title = f.tool === "—" ? "" : f.tool;
  x.ctx.set(f.ctx ?? 0);
  setText(x.ctxValue, f.ctx === null ? "—" : `${Math.round(f.ctx)}%`);
  setText(x.cwd, f.cwd);
  if (x.end) x.end.parentElement!.hidden = !r.session.alive;
  x.arc(r.repo);
  x.locks(r.repo);
  x.shelf(r.repo?.tickets);
  x.pick?.(r.session);
}

function fillTrail(r: Row, entries: readonly TrailEntry[] | undefined, rules?: RowRules) {
  const ruled = (entries ?? []).filter((e) => e.outcome === "auto-allowed" && e.rule);
  const key = `${trailKey(entries)}|${rules ? ruled.map((e) => Number(rules.live(e.rule!))).join("") : ""}`;
  if (key === r.facts.trailKey) return;
  r.facts.trailKey = key;
  const host = r.facts.trail;
  host.replaceChildren();
  host.hidden = !entries?.length;
  const now = Date.now();
  for (const e of entries ?? []) {
    const rule = rules && e.outcome === "auto-allowed" ? e.rule : undefined;
    const item = rule ? button("trail-item is-rule", host) : el("div", "trail-item", host);
    item.dataset.outcome = e.outcome;
    item.title = entryTitle(e, now);
    el("span", "trail-mark", item).textContent = markOf(e.outcome);
    el("span", "trail-tool", item).textContent = e.tool;
    el("span", "trail-target", item).textContent = shownTarget(e);
    if (!rule || !rules) continue;
    const live = rules.live(rule);
    el("span", "trail-revoke", item).textContent = live ? "revoke" : "revoked";
    if (!live) {
      item.classList.add("is-revoked");
      (item as HTMLButtonElement).disabled = true;
      item.title += "\nRule revoked";
      continue;
    }
    item.title += "\nClick to revoke this rule";
    item.addEventListener("click", () => {
      (item as HTMLButtonElement).disabled = true;
      rules.revoke(rule).catch((err) => {
        console.error("peepthis: revoke_rule failed", err);
        (item as HTMLButtonElement).disabled = false;
      });
    });
  }
}

function expand(r: Row, trail?: readonly TrailEntry[], rules?: RowRules) {
  r.closed.hidden = true;
  r.head.hidden = false;
  r.stage.hidden = false;
  r.stage.insertBefore(r.peepWrap, r.stage.firstChild);
  r.peep.setAttribute("size", PEEP_SIZE_OPEN);
  r.nudge.after(r.facts.el);
  r.facts.trailKey = "-";
  fillTrail(r, trail, rules);
  fillFacts(r, Date.now());
  r.el.classList.add("is-open");
}

function collapse(r: Row, restoreFocus: boolean) {
  r.facts.el.remove();
  r.closed.insertBefore(r.peepWrap, r.closed.firstChild);
  r.peep.setAttribute("size", PEEP_SIZE);
  r.head.hidden = true;
  r.stage.hidden = true;
  r.closed.hidden = false;
  r.el.classList.remove("is-open");
  r.el.classList.add("closing");
  setTimeout(() => r.el.classList.remove("closing"), MOTION_MS);
  if (restoreFocus) r.closed.focus();
}

function enter(row: HTMLElement, delayMs: number) {
  row.style.setProperty("--enter-delay", `${delayMs}ms`);
  row.classList.add("entering");
  // A timer, not animationend: the waiting pulse overrides row-enter, which would then never end.
  setTimeout(() => row.classList.remove("entering"), delayMs + MOTION_MS);
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
  rules?: RowRules,
  onEnd?: EndSession,
  looks?: LookStore,
  onDismissTip?: (sessionId: string) => Promise<unknown>,
) {
  const list = el("div", "list", root);
  const empty = el("div", "empty", root);
  el("div", "empty-stage", empty).appendChild(createHousePeep(HOUSE_SIZE_PANEL));
  el("span", "ps-plank empty-plank", empty);
  el("p", "empty-text", empty).textContent = HOUSE_TEXT;
  let current: readonly Session[] = [];
  const syncEmpty = () => {
    let fading = 0;
    for (const r of rows.values()) if (r.el.classList.contains("leaving")) fading++;
    empty.hidden = !houseShown(current, fading);
  };
  const rows = new Map<string, Row>();
  let open: Row | null = null;
  let ticker = 0;
  let foldTimer = 0;
  let firstRender = true;

  function close(restoreFocus = false) {
    if (!open) return;
    collapse(open, restoreFocus);
    open = null;
    clearInterval(ticker);
    ticker = 0;
  }

  let bracketed: Row[] = [];
  /** ADR 0002: the bracket runs plank to plank, so it follows each plank's offset. */
  const placeBrackets = () => {
    for (const r of bracketed) r.el.style.setProperty("--plank-y", `${r.plank.offsetTop}px`);
  };

  function toggle(r: Row) {
    const wasOpen = open === r;
    close(wasOpen && r.head.contains(document.activeElement));
    placeBrackets();
    if (wasOpen) return;
    expand(r, trailOf?.(r.session.id), rules);
    placeBrackets();
    open = r;
    if (r.closed.contains(document.activeElement) || document.activeElement === document.body) r.head.focus();
    ticker = window.setInterval(() => open && fillFacts(open, Date.now()), TICK_MS);
  }

  // Copy before focusing: the clipboard refuses a write once the terminal has taken focus.
  const jumpFor =
    (id: string): Jump =>
    async (cmd) => {
      const ok = await (navigator.clipboard?.writeText(cmd).then(() => true, () => false) ?? false);
      const r = rows.get(id);
      if (onFocus && r) void requestFocus(onFocus, id, () => flash(r.el));
      return ok;
    };

  const snoozed = new Set<string>();
  /** Hides a waved-off tip before the snapshot without it arrives. */
  const dismissed = new Map<string, Tip>();

  async function packUp(id: string, b: HTMLButtonElement) {
    const cmd = jumpCommand("packup");
    const ok = await jumpFor(id)(cmd);
    const label = b.querySelector(".nudge-pack-label")!;
    label.textContent = ok ? "Copied" : `Type ${cmd}`;
    b.classList.toggle("is-copied", ok);
    setTimeout(() => {
      label.textContent = "Pack up";
      b.classList.remove("is-copied");
    }, ok ? COPIED_MS : HINT_MS);
  }

  function showEndError(r: Row, text: string) {
    r.endNote = text;
    r.el.classList.add("is-end-err");
    for (const x of r.says) fillBubble(x, { text });
    clearTimeout(r.endTimer);
    r.endTimer = window.setTimeout(() => {
      r.endNote = undefined;
      r.el.classList.remove("is-end-err");
      if (r.last.bubble) for (const x of r.says) fillBubble(x, r.last.bubble);
    }, HINT_MS);
  }

  function bindEnd(r: Row, b: HTMLButtonElement) {
    let ending = false;
    const settle = () => {
      ending = false;
      b.disabled = false;
      b.textContent = END_LABEL;
    };
    bindHold(
      b,
      async () => {
        ending = true;
        b.disabled = true;
        b.textContent = "Ending…";
        try {
          await onEnd!(r.session.id);
          // Ended: the lifecycle takes the row to done; settle only if it is still here.
          setTimeout(settle, HINT_MS);
        } catch (err) {
          console.error("peepthis: end_session failed", err);
          showEndError(r, `Could not end: ${err instanceof Error ? err.message : String(err)}`);
          settle();
        }
      },
      () => !ending,
    );
  }

  const rowOf = (e: Event) => {
    const id = (e.target as Element).closest<HTMLElement>(".ps-item")?.dataset.id;
    return id ? rows.get(id) : undefined;
  };

  list.addEventListener("click", (e) => {
    const r = rowOf(e);
    if (!r || r.el.classList.contains("leaving")) return;
    const target = e.target as Element;
    if (onFocus && target.closest(".ps-peep, .card-terminal")) {
      void requestFocus(onFocus, r.session.id, () => flash(r.el));
      return;
    }
    if (target.closest(".tip-ok")) {
      if (r.session.tip) dismissed.set(r.session.id, r.session.tip);
      void onDismissTip?.(r.session.id);
      render(current);
      return;
    }
    if (target.closest(".nudge-later")) {
      snoozed.add(r.session.id);
      render(current);
      return;
    }
    const pack = target.closest<HTMLButtonElement>(".nudge-pack");
    if (pack) {
      void packUp(r.session.id, pack);
      return;
    }
    if (target.closest(".ps-row, .ps-openhead")) toggle(r);
  });

  let repos: readonly Repo[] = [];
  let limit: LimitWarning | null = null;
  /** `nextRepos` and `nextLimits` omitted keep the last ones, for re-renders that only carry sessions; null limits clear. */
  function render(sessions: readonly Session[], nextRepos?: readonly Repo[], nextLimits?: RateLimits | null) {
    if (nextRepos) repos = nextRepos;
    if (nextLimits !== undefined) limit = limitWarning(nextLimits ?? undefined);
    const sorted = sortSessions(sessions);
    const tags = clashTags(sorted);
    const seen = new Set<string>();
    const now = Date.now();
    let foldAt: number | null = null;
    let prev: Element | null = null;
    for (const s of sorted) {
      seen.add(s.id);
      let r = rows.get(s.id);
      if (!r) {
        r = createRow(s, jumpFor(s.id), onDecide, rules, !!onEnd, looks);
        if (r.facts.end) bindEnd(r, r.facts.end);
        rows.set(s.id, r);
        enter(r.el, firstRender ? Math.min(seen.size - 1, MAX_STAGGER) * STAGGER_MS : 0);
      }
      r.session = s;
      r.repo = repos.find((p) => p.sessions.includes(s.id));
      if (r.el.classList.contains("leaving")) {
        clearTimeout(Number(r.el.dataset.leaveTimer));
        r.el.classList.remove("leaving");
      }
      if (s.mood !== "yawn") snoozed.delete(s.id);
      if (s.tip !== dismissed.get(s.id)) dismissed.delete(s.id);
      const view = toRowView(s, identityOf, now, snoozed.has(s.id), tags.get(s.id), limit, dismissed.get(s.id));
      updateRow(r, view);
      const kids = r.kids(s, view.color, now);
      const shellFold = r.shells(s, now).foldAt;
      for (const at of [kids.foldAt, shellFold]) if (at !== null && (foldAt === null || at < foldAt)) foldAt = at;
      r.cards?.(s.alive ? s.pending?.filter((p) => !kids.claimed.has(p.id)) : undefined);
      r.offers?.(s.alive && rules ? rules.offersOf(s.id) : []);
      if (r === open) {
        fillFacts(r, now);
        fillTrail(r, trailOf?.(s.id), rules);
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
          syncEmpty();
        }, motionMs()),
      );
    }
    const down = clashLinks(sorted, tags);
    bracketed = [];
    sorted.forEach((s, i) => {
      const r = rows.get(s.id)!;
      const up = i > 0 && down[i - 1];
      r.el.classList.toggle("clash-down", down[i]);
      r.el.classList.toggle("clash-up", up);
      if (down[i] || up) bracketed.push(r);
    });
    placeBrackets();
    current = sessions;
    syncEmpty();
    if (sorted.length > 0) firstRender = false;
    // Finished children fold on time even when no snapshot arrives.
    clearTimeout(foldTimer);
    if (foldAt !== null) foldTimer = window.setTimeout(() => render(sessions), Math.max(0, foldAt - Date.now()) + 50);
  }
  return render;
}
