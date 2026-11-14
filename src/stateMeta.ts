import type { State } from "./model";

// Labels from the design's LABEL map (design/shell/shell.js); colours live in styles.css as --st-*.
export const STATE_LABEL: Record<State, string> = {
  idle: "Idle",
  thinking: "Thinking",
  working: "Working",
  waiting: "Needs you",
  success: "Done",
  error: "Error",
  stuck: "Stuck",
};

export const ATTENTION: readonly State[] = ["waiting", "error", "stuck"];

export const isAttention = (state: State) => ATTENTION.includes(state);
