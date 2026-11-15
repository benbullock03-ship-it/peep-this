/** AV-43: the house peep that stands in for an empty fleet (spec §12.8). It is never a session. */
import type { Session } from "./model";

export const HOUSE_SIZE_STRIP = "40";
export const HOUSE_SIZE_PANEL = "96";
export const HOUSE_TEXT = "No sessions running";

/** Ended sessions still in the snapshot and rows still fading out both keep the house away. */
export function houseShown(sessions: readonly Session[], fading = 0): boolean {
  return sessions.length === 0 && fading === 0;
}

export function createHousePeep(size: string, hover = true): HTMLElement {
  const peep = document.createElement("peep-bot");
  peep.setAttribute("state", "idle");
  peep.setAttribute("color", "yellow");
  peep.setAttribute("size", size);
  if (!hover) peep.setAttribute("hover", "off");
  peep.setAttribute("aria-hidden", "true");
  return peep;
}
