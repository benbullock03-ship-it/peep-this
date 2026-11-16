import { COLORS } from "./identity";
import { pick, type LookStore } from "./look";
import type { Session } from "./model";

// Copied from peep.js `PALETTE`, for the swatches only; the peep itself still takes the name.
const SWATCH: Record<(typeof COLORS)[number], string> = {
  yellow: "#F7D64A",
  pink: "#F7A8C8",
  lavender: "#C2A7EE",
  blue: "#8DC7F2",
  green: "#A5DC93",
};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  parent?.appendChild(e);
  return e;
}

function btn(cls: string, parent: HTMLElement): HTMLButtonElement {
  const b = el("button", cls, parent);
  b.type = "button";
  return b;
}

/** AV-113: the open row's look picker. Each pick applies at once, so the 96px stage is the preview. */
export function createPicker(looks: LookStore) {
  const box = el("div", "ps-pick");
  const toggle = btn("ps-pick-toggle", box);
  toggle.textContent = "Change peep";
  toggle.setAttribute("aria-expanded", "false");
  const body = el("div", "ps-pick-body", box);
  body.hidden = true;
  toggle.addEventListener("click", () => {
    body.hidden = !body.hidden;
    toggle.setAttribute("aria-expanded", String(!body.hidden));
  });

  const colors = el("div", "ps-pick-colors", body);
  colors.setAttribute("role", "group");
  colors.setAttribute("aria-label", "Colour");
  const swatches = COLORS.map((c) => {
    const b = btn("ps-swatch", colors);
    b.dataset.value = c;
    b.style.setProperty("--sw", SWATCH[c]);
    b.title = c;
    b.setAttribute("aria-label", c);
    return b;
  });
  const accs = el("div", "ps-pick-accs", body);
  accs.setAttribute("role", "group");
  accs.setAttribute("aria-label", "Accessory");
  let cells: { b: HTMLButtonElement; peep: HTMLElement }[] = [];
  let shownAccs = "";
  const reset = btn("card-btn ps-pick-reset", body);
  reset.textContent = "Reset to project look";
  reset.title = "Go back to the look this project's folder gives it";

  let session: Session | null = null;
  const apply = (field: "color" | "accessory", value: string) => {
    if (!session) return;
    looks.set(session, pick(looks.baseOf(session), looks.lookOf(session), field, value));
  };
  colors.addEventListener("click", (e) => {
    const v = (e.target as Element).closest<HTMLElement>(".ps-swatch")?.dataset.value;
    if (v) apply("color", v);
  });
  accs.addEventListener("click", (e) => {
    const v = (e.target as Element).closest<HTMLElement>(".ps-acc")?.dataset.value;
    if (v) apply("accessory", v);
  });
  reset.addEventListener("click", () => session && looks.set(session, null));

  function buildAccs(list: readonly string[]) {
    const key = list.join(",");
    if (key === shownAccs) return;
    shownAccs = key;
    accs.replaceChildren();
    cells = list.map((a) => {
      const b = btn("ps-acc", accs);
      b.dataset.value = a;
      b.title = a;
      b.setAttribute("aria-label", a);
      const peep = document.createElement("peep-bot");
      for (const [k, v] of [["state", "idle"], ["accessory", a], ["size", "32"], ["hover", "off"], ["energy", "1"]]) peep.setAttribute(k, v);
      peep.setAttribute("aria-hidden", "true");
      b.appendChild(peep);
      return { b, peep };
    });
  }

  function fill(s: Session) {
    session = s;
    buildAccs(looks.accessories());
    const worn = looks.identityOf(s);
    for (const b of swatches) b.setAttribute("aria-pressed", String(b.dataset.value === worn.color));
    for (const { b, peep } of cells) {
      b.setAttribute("aria-pressed", String(b.dataset.value === worn.accessory));
      if (peep.getAttribute("color") !== worn.color) peep.setAttribute("color", worn.color);
    }
    reset.disabled = !looks.lookOf(s);
  }

  return { el: box, fill };
}
