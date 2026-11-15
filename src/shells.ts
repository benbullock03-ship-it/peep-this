import { formatDuration } from "./details";
import type { Shell } from "./model";

/** An ended shell stays as its own chip this long, then folds, like a finished subagent (KID_LINGER_MS). */
export const SHELL_LINGER_MS = 30_000;
/** IBM Plex Mono advances 600/1000 em per glyph. */
export const MONO_EM = 0.6;

export const isEnded = (sh: Shell) => sh.endedAt !== undefined;
export const isShellFolded = (sh: Shell, now: number) => sh.endedAt !== undefined && now - sh.endedAt >= SHELL_LINGER_MS;

/** Keeps both ends of `text` around one ellipsis, so `npm run dev -- --port 5173` stays recognisable. */
export function middleTruncate(text: string, max: number): string {
  const chars = Array.from(text);
  if (chars.length <= max) return text;
  if (max <= 1) return "…";
  const tail = Math.floor((max - 1) / 2);
  const head = max - 1 - tail;
  return `${chars.slice(0, head).join("")}…${tail ? chars.slice(-tail).join("") : ""}`;
}

/** Characters of a monospace font at `fontPx` that fit in `widthPx`. */
export function charsThatFit(widthPx: number, fontPx: number): number {
  return Math.max(0, Math.floor(widthPx / (fontPx * MONO_EM) + 1e-9));
}

export function shellRuntime(sh: Shell, now: number): string {
  return formatDuration((sh.endedAt ?? now) - sh.startedAt);
}

export function shellLabel(sh: Shell): string {
  switch (sh.state) {
    case "serving":
      return sh.port ? `Serving :${sh.port}` : "Serving";
    case "failed":
      return sh.exitCode !== undefined ? `Failed ${sh.exitCode}` : "Failed";
    case "exited":
      return "Exited";
    case "quiet":
      return "Quiet";
    default:
      return "Running";
  }
}

/** The chip's hover text: the whole command, its description, and how it ended. */
export function shellTitle(sh: Shell): string {
  const lines = [sh.command];
  if (sh.description && sh.description !== sh.command) lines.push(sh.description);
  if (sh.exitCode !== undefined) lines.push(`exit code ${sh.exitCode}`);
  if (sh.lastLine) lines.push(`last: ${sh.lastLine}`);
  return lines.join("\n");
}

export type ShellPlan = { shown: Shell[]; ended: number };

/** Live shells oldest first, then ended ones newest first; ended ones past their linger only when opened. */
export function planShells(shells: readonly Shell[] | undefined, now: number, showEnded = false): ShellPlan {
  const all = shells ?? [];
  const live = all.filter((sh) => !isEnded(sh)).sort((a, b) => a.startedAt - b.startedAt || a.id.localeCompare(b.id));
  const newestFirst = (a: Shell, b: Shell) => b.endedAt! - a.endedAt! || a.id.localeCompare(b.id);
  const ended = all.filter(isEnded);
  const lingering = ended.filter((sh) => !isShellFolded(sh, now)).sort(newestFirst);
  const folded = ended.filter((sh) => isShellFolded(sh, now)).sort(newestFirst);
  return { shown: [...live, ...lingering, ...(showEnded ? folded : [])], ended: folded.length };
}

export function nextShellFoldAt(shells: readonly Shell[] | undefined, now: number): number | null {
  let next: number | null = null;
  for (const sh of shells ?? []) {
    if (sh.endedAt === undefined || isShellFolded(sh, now)) continue;
    const at = sh.endedAt + SHELL_LINGER_MS;
    if (next === null || at < next) next = at;
  }
  return next;
}

/** Hover text for the parent's red dot; empty when no shell failed. Outlives the failed chip's fold. */
export function failNote(shells: readonly Shell[] | undefined): string {
  const failed = (shells ?? [])
    .filter((sh) => sh.state === "failed")
    .sort((a, b) => (b.endedAt ?? b.stateSince) - (a.endedAt ?? a.stateSince));
  return failed
    .map((sh) => {
      const code = sh.exitCode !== undefined ? `exit code ${sh.exitCode}` : "no exit code";
      return [`Shell failed: ${sh.command || sh.id}`, code, sh.lastLine ? `last: ${sh.lastLine}` : ""].filter(Boolean).join("\n");
    })
    .join("\n\n");
}
