import { byOldest, SHELL_TOOLS, targetOf } from "./approval";
import type { PendingApproval, Session } from "./model";

/** Mirrors `STUCK_AFTER_MS` in state.rs: a session turns stuck this long after its last event. */
export const STUCK_AFTER_MS = 6 * 60 * 1000;

// Placeholder tasks state.rs writes when it has nothing better to say.
const DEFAULT_TASKS = new Set(["", "Idle", "Thinking", "Working", "Waiting for input", "Done", "Error"]);

const meaningful = (task: string) => !DEFAULT_TASKS.has(task.trim());

function askFor(p: PendingApproval): string {
  const target = SHELL_TOOLS.has(p.tool) ? String(p.input.command ?? "") : targetOf(p);
  return target ? `Can I use ${p.tool} on ${target}?` : `Can I use ${p.tool}?`;
}

/** The speech-bubble line for a session, from its state and what the snapshot knows. */
export function sayFor(s: Session, now: number): string {
  if (!s.alive) return "Session closed. Bye!";
  const task = s.task ?? "";
  switch (s.state) {
    case "working": {
      const tool = s.lastTool ? [s.lastTool.name, s.lastTool.target].filter(Boolean).join(" ") : meaningful(task) ? task : "";
      return tool ? `Running ${tool}…` : "Working…";
    }
    case "waiting":
      if (meaningful(task)) return task;
      return s.pending?.length ? askFor([...s.pending].sort(byOldest)[0]) : "Waiting for you.";
    case "error":
      return meaningful(task) ? task : "Something went wrong.";
    case "stuck":
      return `zzz… no reply for ${Math.floor((STUCK_AFTER_MS + Math.max(0, now - s.stateSince)) / 60_000)} min`;
    case "success":
      return "All done!";
    case "thinking":
      return meaningful(task) ? task : "Hmm… thinking it over.";
    default:
      return "Resting. Ready when you are.";
  }
}
