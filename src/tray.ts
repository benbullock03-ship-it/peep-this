import { FLOPPIES } from "./bubble";

/**
 * Tray order is the tour's: Core, then Extras. `when` is the tour's "use when" line (walk has none there).
 * Tweet is inserted-only: it seats on the plank but has no tray cell.
 */
export const TRAY: readonly { skill: string; name: string; what: string; when: string }[] = [
  {
    skill: "walk",
    name: "Walk Through",
    what: "Tours the disks in order, asks a few batched questions about your repo and how you work, then sets the pack up to match.",
    when: "Use the first time, or when the pack feels off.",
  },
  {
    skill: "qt",
    name: "Question Time",
    what: "Asks you batched questions about a new idea, reads it back, then writes the tickets.",
    when: "Use when an idea or problem is still fuzzy; ends in tickets.",
  },
  {
    skill: "ticket",
    name: "Ticket",
    what: "Works one ticket end to end: build, check, commit, mark done.",
    when: "Use when a ticket is ready to work, close or park.",
  },
  {
    skill: "shoot",
    name: "Shoot",
    what: "Checks a web UI change in a real browser with DOM asserts and screenshots.",
    when: "Use after a web UI change, before calling it done.",
  },
  {
    skill: "packup",
    name: "Pack Up",
    what: "Writes a short note so a fresh session can carry on from here.",
    when: "Use when stopping, or context is filling up.",
  },
  {
    skill: "ducks",
    name: "Ducks",
    what: "Checks where the repo stands: loose tickets, failing tests, stray changes, the last Pack Up note.",
    when: "Use when sitting down to work, after a Pack Up, or unsure what to pick up next.",
  },
  {
    skill: "sneak",
    name: "Sneak Peep",
    what: "Mocks up two or three versions of a screen on a local server, side by side, before anything is built. You pick one.",
    when: "Use when you want to compare 2 or 3 UI directions before building.",
  },
  {
    skill: "goose",
    name: "Duck Duck Goose",
    what: "Maps what depends on what across the codebase and draws it as a graph.",
    when: "Use when you need to know what calls what before changing it.",
  },
  {
    skill: "coderot",
    name: "Coderot",
    what: "Breaks code on purpose in a throwaway copy to find what the tests miss.",
    when: "Use between tickets, to check the tests can actually fail.",
  },
  {
    skill: "bigpeep",
    name: "Big Peep",
    what: "Runs a batch of tickets, one little peep per ticket, and checks on any that go quiet.",
    when: "Use when several tickets are ready and you want them worked in order, one commit each.",
  },
];

/** FLOPPY-SPEC §02: six disks show; the rest sit behind a `+N` cell. */
export const TRAY_SHOWN = 6;

/** What Jump copies: `/peep:<skill>`, plus an argument such as a ticket id (`/peep:ticket AV-34`). */
export function jumpCommand(skill: string, arg?: string): string {
  const a = arg?.trim();
  return `/peep:${skill}${a ? ` ${a}` : ""}`;
}

/** FLOPPY-SPEC §08: after three Jumps the hint drops the instruction. */
export const hintText = (cmd: string, jumps: number) =>
  jumps > 3 ? `Copied ${cmd}` : `Copied ${cmd} · now paste it in the terminal`;

const DISK_BOX_KEY = "peepthis.diskBox.dismissed";
const HOVER_MS = 400;
const COPIED_MS = 1200;
const HINT_MS = 6000;

let diskBoxDismissed = (() => {
  try {
    return localStorage.getItem(DISK_BOX_KEY) === "1";
  } catch {
    return false;
  }
})();
let jumps = 0;

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

function disk(skill: string, size: 16 | 32 | 48, parent: HTMLElement) {
  const d = el("span", `fl fl-${size}`, parent);
  d.style.setProperty("--col", String(FLOPPIES[skill].col));
  d.setAttribute("aria-hidden", "true");
  return d;
}

function button(cls: string, parent: HTMLElement) {
  const b = el("button", cls, parent);
  b.type = "button";
  return b;
}

function diskBox(onDone: () => void): HTMLElement {
  const box = el("section", "diskbox");
  box.setAttribute("aria-label", "Your disk box");
  el("div", "diskbox-bar", box).textContent = "Peep This · first time here";
  el("h3", "diskbox-title", box).textContent = "Your disk box";
  el("p", "diskbox-text", box).textContent =
    "Each floppy is a skill. Pop one in and the command is copied, ready to paste in that session's terminal.";
  const shelf = el("ol", "diskbox-shelf", box);
  // The tour numbers the five after Walk; Walk sets the pack up first.
  const flow = TRAY.slice(1, TRAY_SHOWN);
  for (const f of flow) {
    const li = el("li", "diskbox-disk", shelf);
    disk(f.skill, 48, li);
    el("span", "diskbox-n", li).textContent = String(shelf.children.length);
  }
  el("p", "diskbox-text", box).textContent = `A good order: ${flow.map((f) => f.name).join(", ")}.`;
  const walk = el("p", "diskbox-rot", box);
  disk("walk", 32, walk);
  el("span", "", walk).textContent = "Walk Through is for the first time, or when the pack feels off.";
  const extra = TRAY.slice(TRAY_SHOWN);
  const more = el("p", "diskbox-rot", box);
  for (const f of extra) disk(f.skill, 32, more);
  el("span", "", more).textContent = `${extra.map((f) => f.name).join(", ")} sit behind +${extra.length} in the tray.`;
  const ok = button("diskbox-ok", box);
  ok.textContent = "Got it";
  ok.addEventListener("click", () => {
    diskBoxDismissed = true;
    try {
      localStorage.setItem(DISK_BOX_KEY, "1");
    } catch {
      // Then it shows again next launch.
    }
    onDone();
  });
  return box;
}

/** Copies the command; resolves false when the clipboard refused. */
export type Jump = (command: string) => Promise<boolean>;

/**
 * The open row's tray section (FLOPPY-SPEC §02, §08, §10): disk box on first run, the tray as one roving tab stop,
 * the Pack Up action, the hover card and the hint bar. `clear()` drops the hint when the session changes state;
 * `fire()` lets the ticket shelf Jump through the same hint bar.
 */
export function createTray(jump: Jump) {
  const sec = el("div", "tray");
  if (!diskBoxDismissed) sec.appendChild(diskBox(() => document.querySelectorAll(".diskbox").forEach((b) => b.remove())));
  el("span", "ps-k tray-k", sec).textContent = "Floppies";
  const row = el("div", "tray-row", sec);
  const group = el("div", "tray-disks", row);
  group.setAttribute("role", "toolbar");
  group.setAttribute("aria-label", "Floppies: copy a skill command for this session");
  const card = el("div", "tray-card", row);
  card.id = `tray-card-${Math.random().toString(36).slice(2, 8)}`;
  card.setAttribute("role", "tooltip");
  card.hidden = true;
  const hint = el("div", "tray-hint", sec);
  hint.setAttribute("role", "status");
  hint.setAttribute("aria-live", "polite");

  let hoverTimer = 0;
  let hintTimer = 0;
  const showCard = (b: HTMLElement, f: (typeof TRAY)[number]) => {
    clearTimeout(hoverTimer);
    hoverTimer = window.setTimeout(() => {
      // Hangs under whichever block holds the disk, so it never covers the open overflow grid.
      b.closest(".tray-row, .tray-more")!.appendChild(card);
      card.replaceChildren();
      disk(f.skill, 48, card);
      const text = el("div", "tray-card-text", card);
      el("strong", "", text).textContent = f.name;
      el("span", "", text).textContent = f.what;
      el("span", "tray-card-when", text).textContent = f.when;
      card.hidden = false;
      b.setAttribute("aria-describedby", card.id);
    }, HOVER_MS);
  };
  const hideCard = (b: HTMLElement) => {
    clearTimeout(hoverTimer);
    card.hidden = true;
    b.removeAttribute("aria-describedby");
  };
  const clear = () => {
    clearTimeout(hintTimer);
    hint.replaceChildren();
  };

  const fire = async (b: HTMLButtonElement, cmd: string) => {
    hideCard(b);
    const ok = await jump(cmd);
    clear();
    if (ok) {
      jumps++;
      el("span", "", hint).textContent = hintText(cmd, jumps);
      if (jumps <= 3) el("kbd", "", hint).textContent = "Ctrl+V";
      b.classList.add("is-copied");
      setTimeout(() => b.classList.remove("is-copied"), COPIED_MS);
    } else {
      el("span", "", hint).textContent = `Couldn't copy. Type ${cmd} in the terminal.`;
    }
    hintTimer = window.setTimeout(clear, HINT_MS);
  };

  /** One roving tab stop per group: arrows move, Home/End jump. */
  const diskGroup = (list: typeof TRAY, parent: HTMLElement) => {
    const disks = list.map((f, i) => {
      const b = button("tray-disk", parent);
      b.tabIndex = i === 0 ? 0 : -1;
      b.setAttribute("aria-label", `${f.name}: copy ${jumpCommand(f.skill)}`);
      b.dataset.skill = f.skill;
      disk(f.skill, 32, b);
      b.addEventListener("pointerenter", () => showCard(b, f));
      b.addEventListener("pointerleave", () => hideCard(b));
      b.addEventListener("focus", () => showCard(b, f));
      b.addEventListener("blur", () => hideCard(b));
      b.addEventListener("click", () => void fire(b, jumpCommand(f.skill)));
      return b;
    });
    parent.addEventListener("keydown", (e) => {
      const at = disks.indexOf(document.activeElement as HTMLButtonElement);
      if (at < 0) return;
      const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
      const to = e.key === "Home" ? 0 : e.key === "End" ? disks.length - 1 : step === undefined ? -1 : (at + step + disks.length) % disks.length;
      if (to < 0) return;
      e.preventDefault();
      disks[at].tabIndex = -1;
      disks[to].tabIndex = 0;
      disks[to].focus();
    });
  };
  diskGroup(TRAY.slice(0, TRAY_SHOWN), group);

  const extra = TRAY.slice(TRAY_SHOWN);
  if (extra.length) {
    const moreBtn = button("tray-disk tray-more-btn", row);
    moreBtn.textContent = `+${extra.length}`;
    moreBtn.setAttribute("aria-label", `${extra.length} more floppies`);
    moreBtn.setAttribute("aria-expanded", "false");
    const moreBox = el("div", "tray-more");
    row.after(moreBox);
    moreBox.id = `${card.id}-more`;
    moreBox.setAttribute("role", "toolbar");
    moreBox.setAttribute("aria-label", "More floppies");
    moreBox.hidden = true;
    moreBtn.setAttribute("aria-controls", moreBox.id);
    diskGroup(extra, moreBox);
    moreBtn.addEventListener("click", () => {
      moreBox.hidden = !moreBox.hidden;
      moreBtn.setAttribute("aria-expanded", String(!moreBox.hidden));
    });
  }

  const pack = button("tray-pack", row);
  disk("packup", 16, pack);
  el("span", "tray-pack-label", pack).textContent = "Pack up";
  pack.setAttribute("aria-label", `Pack up: copy ${jumpCommand("packup")}`);
  pack.addEventListener("click", () => void fire(pack, jumpCommand("packup")));

  return { el: sec, clear, fire };
}
