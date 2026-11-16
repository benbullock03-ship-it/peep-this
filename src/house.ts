/** AV-43: the house peep that stands in for an empty fleet (spec §12.8). It is never a session. */
import type { Session, Snapshot } from "./model";

export const HOUSE_SIZE_STRIP = "40";
export const HOUSE_SIZE_PANEL = "96";
export const HOUSE_TEXT = "No sessions running";
export const START_CMD = "claude";

/** AV-129: the house bubble's line and action; an outdated integration is installed (the banner offers the update). */
export function houseSign(integration?: Snapshot["integration"]): { text: string; action?: "install" | "copy" } {
  if (integration === "missing") return { text: "Install the integration", action: "install" };
  if (integration) return { text: "Start Claude Code in a project", action: "copy" };
  return { text: HOUSE_TEXT };
}

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
