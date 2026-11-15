import type { Session, State } from "./model";
import { toRowView, type IdentityOf } from "./rowView";
import { sortSessions } from "./sort";

/** Spec §12.5: this many peeps, then a `+N` tile. */
export const STRIP_MAX = 8;

export type StripCell = {
  id: string;
  name: string;
  state: State;
  label: string;
  attention: boolean;
  color: string;
  accessory: string;
  alive: boolean;
  /** AV-30 hook: one of the session's background shells failed. */
  shellFailed: boolean;
};

export type StripView = { cells: StripCell[]; more: number };

/** Top-level sessions in the panel's order; subagents and shells never get a cell. */
export function stripView(sessions: readonly Session[], identityOf?: IdentityOf, max = STRIP_MAX): StripView {
  const sorted = sortSessions(sessions);
  const cells = sorted.slice(0, max).map((s) => {
    const { id, name, state, label, attention, color, accessory, alive } = toRowView(s, identityOf);
    return { id, name, state, label, attention, color, accessory, alive, shellFailed: s.shells?.some((sh) => sh.state === "failed") ?? false };
  });
  return { cells, more: sorted.length - cells.length };
}
