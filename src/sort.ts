import type { Session } from "./model";

export function sortSessions(sessions: readonly Session[]): Session[] {
  return [...sessions].sort((a, b) => {
    const wa = a.state === "waiting" ? 0 : 1;
    const wb = b.state === "waiting" ? 0 : 1;
    if (wa !== wb) return wa - wb;
    if (a.stateSince !== b.stateSince) return b.stateSince - a.stateSince;
    return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });
}
