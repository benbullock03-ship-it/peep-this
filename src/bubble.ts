import type { LimitWarning } from "./limits";
import type { Session } from "./model";

/** Peep skill -> floppy art column in the sprite sheet and its verb (FLOPPY-SPEC §01, §05). Pack Up keeps the `handoff` art. */
export const FLOPPIES: Record<string, { col: number; verb: string }> = {
  ducks: { col: 0, verb: "ducking" },
  qt: { col: 1, verb: "questioning" },
  ticket: { col: 2, verb: "ticketing" },
  shoot: { col: 3, verb: "shooting" },
  packup: { col: 4, verb: "packing up" },
  coderot: { col: 5, verb: "rotting" },
  walk: { col: 11, verb: "walking through" },
  sneak: { col: 13, verb: "mocking up" },
  scrub: { col: 14, verb: "scrubbing" },
  bigpeep: { col: 15, verb: "running a batch" },
  tweet: { col: 16, verb: "tweeting" },
  goose: { col: 17, verb: "mapping" },
};
export const BLANK_COL = 6;

/** `id` is the ticket id; `tab` shows it as the attention tab instead of `id:`; `bolt` is a clash's tag letter. */
export type Bubble = { id?: string; text: string; verb?: string; tab?: boolean; bolt?: string };

/** A 4 x 6 pixel bolt drawn at 2px per cell. */
const BOLT_PATH = "M2 0h2v1h-2zM1 1h2v1h-2zM0 2h4v1h-4zM1 3h2v1h-2zM0 4h2v1h-2zM0 5h1v1h-1z";

/**
 * FLOPPY-SPEC §05: `ID: title · verb`; an attention state's ask replaces the title and drops the verb.
 * AV-79 §04: a rate-limit `limit` takes the verb slot, or the whole line when there is no ticket.
 */
export function bubbleFor(
  s: Pick<Session, "alive" | "skill" | "ticket">,
  say: string,
  attention: boolean,
  limit?: LimitWarning | null,
): Bubble {
  if (!s.alive) return { text: say };
  const id = s.ticket?.id;
  if (attention) return id ? { id, text: say, tab: true } : { text: say };
  if (limit && !id) return { text: `${limit.key} limit ${limit.pct}%` };
  const verb = limit ? `${limit.key} ${limit.pct}%` : s.skill ? (FLOPPIES[s.skill]?.verb ?? s.skill) : undefined;
  return { id, text: (id && s.ticket?.title?.trim()) || say, verb };
}

export const bubbleLine = (b: Bubble) => `${b.id ? `${b.id}: ` : ""}${b.text}${b.verb ? ` · ${b.verb}` : ""}`;

/** Fills the `.ps-bubble > span`; a bubble without ticket or verb stays plain text as before AV-66. */
export function fillBubble(span: HTMLElement, b: Bubble) {
  span.parentElement!.title = bubbleLine(b);
  if (!b.id && !b.verb && !b.bolt) {
    span.className = "";
    span.textContent = b.text;
    return;
  }
  span.className = b.tab ? "bb is-tab" : "bb";
  span.replaceChildren();
  const part = (cls: string, text: string) => {
    const e = document.createElement("span");
    e.className = cls;
    e.textContent = text;
    span.appendChild(e);
    return e;
  };
  if (b.bolt) {
    const tag = part("bb-bolt", b.bolt);
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 4 6");
    svg.setAttribute("aria-hidden", "true");
    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("d", BOLT_PATH);
    svg.appendChild(path);
    tag.prepend(svg);
  }
  if (b.id) part("bb-id", b.id);
  part("bb-text", b.text);
  if (b.verb) part("bb-verb", b.verb);
}
