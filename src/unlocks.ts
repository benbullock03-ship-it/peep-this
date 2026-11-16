/** tickets.rs `Unlock`. */
export type Unlock = { ticketRuns: number; closedUi: boolean; test: boolean; url: boolean };

/** What the rules read off a snapshot `Repo`; `unlock` is absent in the browser demo. */
export type UnlockSource = { arc: { closed: number }; unlock?: Unlock };

type Facts = Unlock & { closed: number };

/** AV-108, the same rules as `plugin/skills/walk/references/tour.md`. Skills without a rule are never locked. */
export const UNLOCKS: Readonly<Record<string, { met: (f: Facts) => boolean; hint: string }>> = {
  bigpeep: { met: (f) => f.ticketRuns >= 5, hint: "unlocks after 5 tickets" },
  coderot: { met: (f) => f.test && f.closed >= 1, hint: "unlocks after your first ticket closed with a test command in setup" },
  goose: { met: (f) => f.closed >= 3, hint: "unlocks after 3 tickets closed" },
  sneak: { met: (f) => f.closedUi, hint: "unlocks after your first UI ticket closed" },
  scrub: { met: (f) => f.closedUi, hint: "unlocks after your first UI ticket closed" },
  shoot: { met: (f) => f.url, hint: "unlocks when the setup has a url" },
};

/** The unlock hint while `skill` is locked, else null; null too with no repo or no unlock facts. */
export function lockHint(skill: string, repo: UnlockSource | undefined): string | null {
  const rule = UNLOCKS[skill];
  if (!rule || !repo?.unlock) return null;
  return rule.met({ ...repo.unlock, closed: repo.arc.closed }) ? null : rule.hint;
}
