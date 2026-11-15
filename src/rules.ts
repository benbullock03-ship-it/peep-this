import type { Rule, RuleAnswer, RuleOffer, RulesView } from "./model";

/** rules.rs `FILES`. */
export const FILES = "<files>";
export const FILES_NOTE = `${FILES} stands for one or more file paths inside the project.`;

export const ANSWERS: readonly { answer: RuleAnswer; label: string }[] = [
  { answer: "yes", label: "Yes" },
  { answer: "not-now", label: "Not now" },
  { answer: "never", label: "Never ask" },
];

export const EMPTY_RULES: RulesView = { rules: [], offers: [] };

export const offersFor = (v: RulesView, sessionId: string): RuleOffer[] => v.offers.filter((o) => o.sessionId === sessionId);

export const ruleLive = (v: RulesView, id: string) => v.rules.some((r) => r.id === id);

/** Shell commands read as the command alone; other tools name themselves. */
export function offerWhat(o: RuleOffer): { tool: string | null; pattern: string } {
  const shell = o.tool === "Bash" || o.tool === "PowerShell";
  return { tool: shell ? null : o.tool, pattern: o.pattern };
}

export const usesText = (r: Rule) => (r.uses === 1 ? "used once" : `used ${r.uses} times`);
