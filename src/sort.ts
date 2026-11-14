import type { Session } from "./model";
import { isAttention } from "./stateMeta";

export function sortSessions(sessions: readonly Session[]): Session[] {
  return [...sessions].sort((a, b) => {
    const att = Number(isAttention(b.state)) - Number(isAttention(a.state));
    if (att !== 0) return att;
    if (a.stateSince !== b.stateSince) return b.stateSince - a.stateSince;
    return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });
}
