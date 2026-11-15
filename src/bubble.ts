import type { Session } from "./model";

/** Peep skill -> floppy art column in the sprite sheet and its verb (FLOPPY-SPEC §01, §05). Pack Up keeps the `handoff` art. */
export const FLOPPIES: Record<string, { col: number; verb: string }> = {
  ducks: { col: 0, verb: "ducking" },
  qt: { col: 1, verb: "questioning" },
  ticket: { col: 2, verb: "ticketing" },
  shoot: { col: 3, verb: "shooting" },
  packup: { col: 4, verb: "packing up" },
  coderot: { col: 5, verb: "rotting" },
};
export const BLANK_COL = 6;

/** `id` is the ticket id; `tab` shows it as the attention tab instead of `id:`. */
export type Bubble = { id?: string; text: string; verb?: string; tab?: boolean };

/** FLOPPY-SPEC §05: `ID: title · verb`; an attention state's ask replaces the title and drops the verb. */
export function bubbleFor(s: Pick<Session, "alive" | "skill" | "ticket">, say: string, attention: boolean): Bubble {
  if (!s.alive) return { text: say };
  const id = s.ticket?.id;
  if (attention) return id ? { id, text: say, tab: true } : { text: say };
  const verb = s.skill ? (FLOPPIES[s.skill]?.verb ?? s.skill) : undefined;
  return { id, text: (id && s.ticket?.title?.trim()) || say, verb };
}

export const bubbleLine = (b: Bubble) => `${b.id ? `${b.id}: ` : ""}${b.text}${b.verb ? ` · ${b.verb}` : ""}`;

/** Fills the `.ps-bubble > span`; a bubble without ticket or verb stays plain text as before AV-66. */
export function fillBubble(span: HTMLElement, b: Bubble) {
  span.parentElement!.title = bubbleLine(b);
  if (!b.id && !b.verb) {
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
  };
  if (b.id) part("bb-id", b.id);
  part("bb-text", b.text);
  if (b.verb) part("bb-verb", b.verb);
}
