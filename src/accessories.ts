import { ACCESSORIES, COLORS } from "./identity";

const POSES = ["idle", "waiting", "error"] as const;

/** Dev-only `?accessories` page: every accessory at 96px in three poses, for the owner's art check. */
export function renderAccessoryCheck(root: HTMLElement) {
  root.classList.add("acc-check");
  ACCESSORIES.forEach((acc, i) => {
    const cell = document.createElement("figure");
    cell.className = "acc-cell";
    for (const pose of POSES) {
      const peep = document.createElement("peep-bot");
      peep.setAttribute("state", pose);
      peep.setAttribute("color", COLORS[i % COLORS.length]);
      peep.setAttribute("accessory", acc);
      peep.setAttribute("size", "96");
      peep.setAttribute("hover", "off");
      cell.appendChild(peep);
    }
    const caption = document.createElement("figcaption");
    caption.textContent = acc;
    cell.appendChild(caption);
    root.appendChild(cell);
  });
}
