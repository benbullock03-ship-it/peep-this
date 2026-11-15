/** AV-41: presses inside a hover peek pin it as `panel`, except these row actions. */
export const KEEPS_PEEK = ".ps-item .ps-peep, .card-allow, .card-deny, #collapse-btn";

export function pinsPeek(target: { closest(selector: string): unknown } | null): boolean {
  return !!target && !target.closest(KEEPS_PEEK);
}
