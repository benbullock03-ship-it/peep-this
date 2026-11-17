import { TIP_TEXT } from "./tip";
import type { Tip } from "./model";

/** learned.rs `Count`; `first` is epoch ms. */
export type RhythmCount = { count: number; first: number | null };
export type Rhythm = { packups: RhythmCount; tickets: RhythmCount; approvals: RhythmCount };

/** learned.rs `Learned`: ids of tips shown, Peep skills run and patterns a Goose health report cited, plus the rhythm counter. */
export type Learned = { tips: string[]; skills: string[]; patterns: string[]; rhythm: Rhythm };

const RHYTHM_LABELS: [keyof Rhythm, string][] = [
  ["packups", "Sessions with a Pack Up"],
  ["tickets", "Tickets closed through /peep:ticket"],
  ["approvals", "Approvals from the widget"],
];

function day(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

export function rhythmLines(r: Rhythm): string[] {
  return RHYTHM_LABELS.map(([k, label]) => {
    const c = r[k];
    return `${label}: ${c.count}${c.first == null ? "" : `, first ${day(c.first)}`}`;
  });
}

/** The plain text "copy my rhythm" puts on the clipboard for a beta issue. */
export function rhythmSummary(r: Rhythm): string {
  return ["Peep This rhythm", ...rhythmLines(r)].join("\n");
}

export const TOPICS = ["context", "rhythm", "testing", "working with sessions", "architecture"] as const;
export type Topic = (typeof TOPICS)[number];

type Source = { label: string; url?: string };
export type Lesson = { kind: "tip" | "skill" | "pattern"; id: string; topic: Topic; line: string; eli5: string; source: Source };

const S = {
  rot: { label: "Chroma, Context Rot", url: "https://www.trychroma.com/research/context-rot" },
  middle: { label: "Liu et al., Lost in the Middle", url: "https://arxiv.org/abs/2307.03172" },
  practices: { label: "Anthropic, Claude Code best practices", url: "https://code.claude.com/docs/en/best-practices" },
  cache: { label: "Anthropic docs, prompt caching", url: "https://platform.claude.com/docs/en/build-with-claude/prompt-caching" },
  subagents: { label: "Anthropic docs, subagents", url: "https://code.claude.com/docs/en/sub-agents" },
  costs: { label: "Anthropic docs, managing costs and /compact", url: "https://code.claude.com/docs/en/costs" },
  mutation: { label: "Mutation testing", url: "https://en.wikipedia.org/wiki/Mutation_testing" },
  ousterhout: { label: "Ousterhout, A Philosophy of Software Design" },
} satisfies Record<string, Source>;

const TIPS: Record<Tip, Omit<Lesson, "kind" | "id" | "line">> = {
  crowded: { topic: "context", eli5: "A desk piled high: the note you need is still there, but harder to find.", source: S.rot },
  misses: { topic: "working with sessions", eli5: "If the key did not fit twice, stop jiggling it and fetch the right key.", source: S.practices },
  cold: { topic: "context", eli5: "The kettle went cold while you were away, so it has to boil from the start.", source: S.cache },
  helper: { topic: "working with sessions", eli5: "You send a friend to read the whole book; they come back with the one page you need.", source: S.subagents },
  compacted: { topic: "context", eli5: "Your notes were squashed onto one card: the headlines stayed, the small print went.", source: S.costs },
};

// The Why lines from each plugin/skills/<id>/SKILL.md; learned.test.ts keeps them in step.
const SKILLS: Record<string, Omit<Lesson, "kind" | "id">> = {
  qt: { topic: "rhythm", line: "ten minutes of questions cost less than rewriting the wrong code.", eli5: "Measure twice, cut once.", source: S.practices },
  ticket: { topic: "rhythm", line: "one ticket finished beats three half-built.", eli5: "One finished sandcastle beats three piles of sand.", source: S.practices },
  packup: { topic: "context", line: "close the session cleanly so the next one starts where this one stopped.", eli5: "Leave a bookmark so tomorrow you open the right page.", source: S.middle },
  ducks: { topic: "rhythm", line: "finish what is loose before you start something new.", eli5: "Put the toys away before getting new ones out.", source: S.practices },
  walk: { topic: "rhythm", line: "a minute of setup now saves every later skill from guessing.", eli5: "Tell the new helper where the cups are once, and they stop opening every cupboard.", source: S.practices },
  bigpeep: { topic: "working with sessions", line: "watch the diffs, not the reports; a report is a claim, the diff is the evidence.", eli5: "Look in the room yourself instead of trusting \"I tidied it\".", source: S.practices },
  tweet: { topic: "working with sessions", line: "a question on the board costs a minute; a clash found at commit costs the afternoon.", eli5: "Ask \"is anyone using the oven?\" before you start baking.", source: S.practices },
  coderot: { topic: "testing", line: "a test that cannot fail proves nothing.", eli5: "A smoke alarm that never beeps, even in smoke, is just a plastic disc.", source: S.mutation },
  shoot: { topic: "testing", line: "a shot you checked proves it works; reading the code does not.", eli5: "Taste the soup instead of reading the recipe again.", source: S.practices },
  sneak: { topic: "testing", line: "seeing three directions costs minutes, rebuilding the wrong one costs a day.", eli5: "Try on three jumpers in the shop rather than knitting the wrong one.", source: S.practices },
  goose: { topic: "architecture", line: "know what a change touches before making it, so you think before code.", eli5: "Check which lights a switch controls before you rewire it.", source: S.ousterhout },
};

// plugin/references/patterns.md; learned.test.ts keeps them in step.
const PATTERNS: Record<string, { name: string; line: string; eli5: string; source: string }> = {
  "shallow-module": { name: "Shallow module", line: "A module whose interface is as big as what it hides costs more to learn than it saves.", eli5: "A box with so many buttons you may as well do the job yourself.", source: "Ousterhout, A Philosophy of Software Design, ch. 4" },
  "scattered-concept": { name: "Scattered concept", line: "One idea spread over many files means every change to it touches all of them.", eli5: "Your socks are in five drawers, so getting dressed takes forever.", source: "Ousterhout ch. 5 (information leakage); Pocock, codebase-design skill" },
  "leaky-seam": { name: "Leaky seam", line: "When callers must know how a module works inside, the seam between them is not doing its job.", eli5: "The kitchen hatch is so small the waiter has to come in and cook.", source: "Parnas, \"On the Criteria To Be Used in Decomposing Systems into Modules\" (1972)" },
  "pass-through": { name: "Pass-through layer", line: "A layer that only forwards calls adds a hop and hides nothing.", eli5: "A messenger who just repeats what you said to the person next to you.", source: "Ousterhout ch. 7 (pass-through methods)" },
  "untested-core": { name: "Untested core", line: "Code many things depend on, with no test that can fail, makes every change a guess.", eli5: "The bridge everyone drives over has never had a safety check.", source: "Feathers, Working Effectively with Legacy Code, ch. 2" },
  "god-file": { name: "God file", line: "One file that owns many unrelated jobs grows until nobody can change it safely.", eli5: "One drawer holding the cutlery, the batteries and the passports.", source: "Riel, Object-Oriented Design Heuristics, heuristic 3.2" },
};

export const SKILL_WHY: Readonly<Record<string, string>> = Object.fromEntries(Object.entries(SKILLS).map(([id, s]) => [id, s.line]));
export const PATTERN_IDS: readonly string[] = Object.keys(PATTERNS);

export const LESSONS: readonly Lesson[] = [
  ...(Object.keys(TIPS) as Tip[]).map((id): Lesson => ({ kind: "tip", id, line: TIP_TEXT[id], ...TIPS[id] })),
  ...Object.entries(SKILLS).map(([id, s]): Lesson => ({ kind: "skill", id, ...s, line: `/peep:${id}: ${s.line}` })),
  ...Object.entries(PATTERNS).map(([id, p]): Lesson => ({ kind: "pattern", id, topic: "architecture", line: `${p.name}: ${p.line}`, eli5: p.eli5, source: { label: p.source } })),
];

/** Seen lessons grouped in topic order, skipping empty topics; `remaining` counts the unseen ones. */
export function learnedView(seen: Learned): { groups: { topic: Topic; lessons: Lesson[] }[]; remaining: number } {
  const has = { tip: new Set(seen.tips), skill: new Set(seen.skills), pattern: new Set(seen.patterns) };
  const shown = LESSONS.filter((l) => has[l.kind].has(l.id));
  return {
    groups: TOPICS.map((topic) => ({ topic, lessons: shown.filter((l) => l.topic === topic) })).filter((g) => g.lessons.length),
    remaining: LESSONS.length - shown.length,
  };
}

export type LearnedApi = { seen: () => Promise<Learned>; open: (url: string) => void };

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent: HTMLElement, text?: string) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text !== undefined) e.textContent = text;
  parent.appendChild(e);
  return e;
}

/** The "What Peep taught you" view; `refresh` rereads what has been seen. */
export function createLearned(host: HTMLElement, api: LearnedApi, onClose: () => void) {
  async function refresh() {
    const seen = await api.seen();
    const view = learnedView(seen);
    host.replaceChildren();
    const head = el("div", "settings-head", host);
    el("h2", "settings-title", head, "What Peep taught you");
    const close = el("button", "settings-close", head, "×");
    close.type = "button";
    close.title = "Back to sessions";
    close.setAttribute("aria-label", "Back to sessions");
    close.addEventListener("click", onClose);
    el("h3", "settings-sub", host, "Your rhythm");
    const rhythm = el("ul", "rhythm-list", host);
    for (const line of rhythmLines(seen.rhythm)) {
      const [count, first] = line.split(", first ");
      const li = el("li", "", rhythm, first === undefined ? line : `${count}, `);
      if (first !== undefined) el("span", "rhythm-date", li, `first ${first}`);
    }
    const copyRow = el("div", "rhythm-copy", host);
    const copy = el("button", "card-btn", copyRow, "Copy my rhythm");
    copy.type = "button";
    const copied = el("span", "settings-note", copyRow);
    copy.addEventListener("click", () => {
      navigator.clipboard.writeText(rhythmSummary(seen.rhythm)).then(
        () => (copied.textContent = "Copied. Stays on this machine until you paste it."),
        () => (copied.textContent = "Couldn't copy."),
      );
    });
    if (!view.groups.length) el("p", "settings-note", host, "Nothing yet. Tips and skills add a line here the first time you meet them.");
    for (const g of view.groups) {
      el("h3", "settings-sub", host, g.topic[0].toUpperCase() + g.topic.slice(1));
      const list = el("ul", "learned-list", host);
      for (const l of g.lessons) {
        const item = el("li", "learned-item", list);
        el("p", "learned-line", item, l.line);
        const fold = el("details", "learned-fold", item);
        el("summary", "", fold, "Explain like I'm 5");
        el("p", "", fold, l.eli5);
        const src = el("p", "learned-source", item);
        if (l.source.url) {
          const a = el("a", "", src, l.source.label);
          a.href = l.source.url;
          a.addEventListener("click", (e) => {
            e.preventDefault();
            api.open(l.source.url!);
          });
        } else src.textContent = l.source.label;
      }
    }
    if (view.remaining) el("p", "settings-note learned-remaining", host, `${view.remaining} more to find as you go.`);
  }
  return { refresh };
}
