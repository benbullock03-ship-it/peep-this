import { formatDuration } from "./details";
import type { DigestEntry, DigestGroup } from "./model";

/** Mirrors `AWAY_AFTER_CHOICES` in src-tauri/src/away.rs. */
export const AWAY_CHOICES = [1, 2, 5, 10, 15, 30] as const;

export const GROUPS: readonly DigestGroup[] = ["blocked", "failed", "finished", "running"];

export const GROUP_LABEL: Record<DigestGroup, string> = {
  blocked: "Blocked on you",
  failed: "Failed",
  finished: "Finished",
  running: "Still running",
};

/** State colour token per group, for the section mark. */
export const GROUP_STATE: Record<DigestGroup, string> = {
  blocked: "waiting",
  failed: "error",
  finished: "success",
  running: "working",
};

/** Consecutive runs in the backend's order, empty groups left out. */
export function sections(entries: readonly DigestEntry[]): { group: DigestGroup; entries: DigestEntry[] }[] {
  return GROUPS.map((group) => ({ group, entries: entries.filter((e) => e.group === group) })).filter((s) => s.entries.length > 0);
}

const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;
const floor = (n: number, partial: boolean) => (partial ? `${n}+` : String(n));

const baseName = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? p;

/** The facts line under an entry's name: time, files, commands, releases. */
export function factsOf(e: DigestEntry): string[] {
  const out = [formatDuration(e.tookMs)];
  if (e.fileCount > 0) out.push(`${floor(e.fileCount, e.partial)} file${e.fileCount === 1 && !e.partial ? "" : "s"} changed`);
  if (e.commands > 0) out.push(`${floor(e.commands, e.partial)} command${e.commands === 1 && !e.partial ? "" : "s"}`);
  if (e.released > 0) out.push(`${plural(e.released, "approval")} went to the terminal`);
  return out;
}

/** Base names of the listed files, the rest counted; full paths go in the title. */
export function filesLine(e: DigestEntry): string {
  if (e.fileCount === 0) return "";
  const names = e.files.map(baseName).join(", ");
  const more = e.fileCount - e.files.length;
  return more > 0 ? `${names} +${more}` : names;
}

/** What a click on the entry does: blocked on a held approval opens the inbox, everything else focuses the terminal. */
export const entryAction = (e: DigestEntry): "inbox" | "focus" | "none" =>
  e.group === "blocked" && e.pending ? "inbox" : e.alive ? "focus" : "none";

export function headline(since: number, until: number, entries: readonly DigestEntry[]): string {
  return `Away ${formatDuration(until - since)} · ${plural(entries.length, "session")}`;
}
