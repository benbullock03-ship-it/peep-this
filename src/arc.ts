import type { Repo } from "./model";
import { jumpCommand } from "./tray";

export type ArcCount = { n: number; label: string; lead?: boolean };

/** SKILL-PACK-2 §05: "N closed · N open · N parked", closed first, zero counts dropped. */
export function arcCounts(arc: Repo["arc"]): ArcCount[] {
  const all: ArcCount[] = [
    { n: arc.closed, label: "closed", lead: true },
    { n: arc.open, label: "open" },
    { n: arc.parked, label: "parked" },
  ];
  return all.filter((c) => c.n > 0);
}

/** The hover/focus text: the full north star, the counts (the line may drop some), then the last ticket closed with its title. */
export function arcTip(repo: Pick<Repo, "northStar" | "arc">): string {
  const last = repo.arc.last;
  const counts = arcCounts(repo.arc).map((c) => `${c.n} ${c.label}`).join(" · ");
  return [
    repo.northStar ?? "No north star yet. Walk Through pins one down.",
    counts,
    last ? `Last closed: ${last.id}${last.title ? ` · ${last.title}` : ""}` : "None closed yet",
  ]
    .filter(Boolean)
    .join("\n");
}

const CLOSING_MS = 200;

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

/** The open row's arc line. `render(undefined)` hides it: the session has no repo yet. */
export function createArc(jump: (b: HTMLButtonElement, command: string) => void) {
  const line = el("div", "arc");
  line.tabIndex = 0;
  let key = "";
  let lastId: string | undefined;
  let root: string | undefined;
  let timer = 0;

  function render(repo: Repo | undefined) {
    const k = repo ? JSON.stringify([repo.root, repo.northStar, repo.arc]) : "";
    if (k === key) return;
    key = k;
    line.hidden = !repo;
    line.replaceChildren();
    if (!repo) {
      root = lastId = undefined;
      return;
    }
    const { arc, northStar } = repo;
    const closing = root === repo.root && arc.last !== undefined && lastId !== undefined && arc.last.id !== lastId;
    root = repo.root;
    lastId = arc.last?.id;

    el("span", "arc-star", line).setAttribute("aria-hidden", "true");
    line.classList.toggle("is-unset", !northStar);
    if (northStar) {
      el("span", "arc-ns", line).textContent = northStar;
    } else {
      el("span", "arc-ns", line).textContent = "No north star yet ·";
      const b = el("button", "arc-jump", line);
      b.type = "button";
      b.textContent = "Walk Through";
      b.setAttribute("aria-label", `Walk Through: copy ${jumpCommand("walk")}`);
      b.addEventListener("click", () => jump(b, jumpCommand("walk")));
    }
    const cnt = el("span", "arc-cnt", line);
    for (const c of arcCounts(arc)) {
      const s = el("span", "", cnt);
      if (c.lead) {
        el("span", "arc-c", s).textContent = String(c.n);
        s.append(` ${c.label}`);
      } else s.textContent = `${c.n} ${c.label}`;
    }
    const last = el("span", arc.last ? "arc-last" : "arc-last is-none", line);
    if (arc.last) {
      el("span", "arc-tick", last).setAttribute("aria-hidden", "true");
      last.append(arc.last.id);
    } else last.textContent = "none closed yet";

    const tip = arcTip(repo);
    el("span", "arc-tip", line).textContent = tip;
    line.setAttribute("aria-label", `Arc: ${tip.replace(/\n/g, ". ")}`);

    clearTimeout(timer);
    line.classList.remove("is-closing");
    if (closing) {
      void line.offsetWidth;
      line.classList.add("is-closing");
      timer = window.setTimeout(() => line.classList.remove("is-closing"), CLOSING_MS);
    }
  }

  return { el: line, render };
}
