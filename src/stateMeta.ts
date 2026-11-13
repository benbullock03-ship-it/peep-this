import type { State } from "./model";

// LED colours and labels from STATE_META in design/peep-this.dc.html.
export const STATE_META: Record<State, { label: string; led: string }> = {
  idle: { label: "Idle", led: "#DDF3FF" },
  thinking: { label: "Thinking", led: "#C7B2FF" },
  working: { label: "Working", led: "#7FE6FF" },
  waiting: { label: "Needs you", led: "#FFB23E" },
  success: { label: "Done", led: "#7DFFA6" },
  error: { label: "Error", led: "#FF5D5D" },
  stuck: { label: "Stuck", led: "#8C97AB" },
};
