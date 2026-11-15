import { describe, expect, it } from "vitest";
import type { Rule, RuleOffer, RulesView } from "./model";
import { ANSWERS, EMPTY_RULES, FILES, offersFor, offerWhat, ruleLive, usesText } from "./rules";
import { entryTitle } from "./trail";

const offer = (over: Partial<RuleOffer> = {}): RuleOffer => ({
  id: "of-1",
  sessionId: "s1",
  cwd: "c:/work/demo",
  tool: "Bash",
  pattern: "npm test",
  tier: "shell",
  at: 1,
  ...over,
});

const rule = (over: Partial<Rule> = {}): Rule => ({ id: "r1", cwd: "c:/work/demo", tool: "Bash", pattern: "npm test", created: 1, uses: 0, ...over });

describe("rule offers", () => {
  it("belong to the session whose allow raised them", () => {
    const v: RulesView = { rules: [], offers: [offer(), offer({ id: "of-2", sessionId: "s2" })] };
    expect(offersFor(v, "s1").map((o) => o.id)).toEqual(["of-1"]);
    expect(offersFor(v, "s3")).toEqual([]);
    expect(offersFor(EMPTY_RULES, "s1")).toEqual([]);
  });

  it("name a shell command alone and other tools with their tool", () => {
    expect(offerWhat(offer())).toEqual({ tool: null, pattern: "npm test" });
    expect(offerWhat(offer({ tool: "PowerShell", pattern: `npx vitest run ${FILES}` }))).toEqual({ tool: null, pattern: "npx vitest run <files>" });
    expect(offerWhat(offer({ tool: "Edit", pattern: "src/*.ts" }))).toEqual({ tool: "Edit", pattern: "src/*.ts" });
  });

  it("answer yes, not now or never ask, in that order", () => {
    expect(ANSWERS.map((a) => a.answer)).toEqual(["yes", "not-now", "never"]);
    expect(ANSWERS.map((a) => a.label)).toEqual(["Yes", "Not now", "Never ask"]);
  });
});

describe("rules", () => {
  it("are live until revoked", () => {
    const v: RulesView = { rules: [rule()], offers: [] };
    expect(ruleLive(v, "r1")).toBe(true);
    expect(ruleLive({ ...v, rules: [] }, "r1")).toBe(false);
  });

  it("count their uses", () => {
    expect(usesText(rule())).toBe("used 0 times");
    expect(usesText(rule({ uses: 1 }))).toBe("used once");
    expect(usesText(rule({ uses: 12 }))).toBe("used 12 times");
  });

  it("an auto-allowed trail entry names its rule", () => {
    const title = entryTitle(
      { t: 0, sessionId: "s1", session: "w", cwd: "C:\\w", tool: "Bash", target: "npm test", tier: "shell", outcome: "auto-allowed", rule: "r1" },
      0,
    );
    expect(title).toContain("auto-allowed · rule r1");
  });
});
