import "./vendor/peep.js";
import { renderAccessoryCheck } from "./accessories";
import { createBanner, type TrayResult } from "./banner";
import { demoDecide, demoDigest, demoIdentity, demoSnapshot, demoStep } from "./demo";
import { flyWants, pruneDismissed, type Dismissed } from "./flyout";
import { boxOf, clearFold, painted, playFold, showBeacon, viewportIs, type Box, type FoldGeometry } from "./fold";
import { createDigestCard } from "./digestCard";
import { createInbox, type InboxDeps } from "./inboxView";
import { createLimits } from "./limits";
import type { Digest, RulesView, Session, Snapshot, TrailEntry } from "./model";
import { EMPTY_RULES, offersFor, ruleLive } from "./rules";
import { reducedMotion } from "./motion";
import { pinsPeek } from "./peek";
import { createList } from "./rows";
import { createSettings, type HotkeyStatus, type SettingsApi } from "./settings";
import { createStrip, slotCenter, type StripActions } from "./stripPlate";
import { initTheme } from "./theme";
import { ROW_TRAIL } from "./trail";
import { watchContent } from "./windowFit";

const DEMO_STEP_MS = 3200;
const theme = initTheme();

/** Mirrors `ModeView` in src-tauri/src/window.rs. `layout` is the visible shape even while hidden. */
type ModeView = { mode: "panel" | "strip" | "hidden"; layout: "panel" | "column" | "row"; anchor: string; peek?: boolean };

const shellEl = document.querySelector<HTMLElement>("#shell")!;
const stripEl = document.querySelector<HTMLElement>("#strip")!;
let view: ModeView = { mode: "panel", layout: "panel", anchor: "float" };
let remeasure = () => {};
const stripActions: StripActions = { open: () => applyMode({ ...view, mode: "panel", layout: "panel" }) };
// Replaced by the backend's switch once Tauri is up; in a browser the swap is local.
let collapse = () => applyMode({ ...view, mode: "strip", layout: "column" });
const collapseBtn = document.querySelector<HTMLButtonElement>("#collapse-btn")!;
collapseBtn.addEventListener("click", () => collapse());

function setShape(v: ModeView) {
  view = v;
  shellEl.hidden = v.layout !== "panel";
  stripEl.hidden = v.layout === "panel";
  stripEl.dataset.layout = v.layout;
  // styles.css pins the card to the anchored side, so an unfold grows away from it.
  document.body.dataset.anchor = v.anchor;
  remeasure();
  syncFlyouts();
}

/** AV-42: tells the backend which waiting rows fly out of the strip, and where their peeps sit on it. */
let flySessions: readonly Session[] = [];
const dismissed: Dismissed = new Map();
let sendFlyouts: ((items: { id: string; at: number }[]) => void) | null = null;
let flyFrame = 0;
let lastFly = "";
function syncFlyouts() {
  if (!sendFlyouts || flyFrame) return;
  flyFrame = requestAnimationFrame(() => {
    flyFrame = 0;
    pruneDismissed(dismissed, flySessions);
    const show = view.mode === "strip" && !view.peek && view.layout !== "panel" && !folding;
    const items = flyWants(flySessions, show, dismissed).flatMap(({ id, slot }) => {
      const at = slotCenter(stripEl, slot);
      return at === null ? [] : [{ id, at: Math.round(at) }];
    });
    const key = JSON.stringify(items);
    if (key === lastFly) return;
    lastFly = key;
    sendFlyouts?.(items);
  });
}

const SHADOW_PX = 6;
const cardBox = (el: HTMLElement): Box => {
  const r = el.getBoundingClientRect();
  return { x: r.left, y: r.top, w: r.width + SHADOW_PX, h: r.height + SHADOW_PX };
};

/** How a switch gets its window. In a browser nothing resizes: both shapes are measured where they lie. */
let surface = {
  begin: async (v: ModeView): Promise<FoldGeometry | null> => {
    const [incoming, outgoing] = v.layout === "panel" ? [shellEl, stripEl] : [stripEl, shellEl];
    const from = cardBox(outgoing);
    incoming.classList.remove("fold-measure");
    outgoing.hidden = true;
    const to = cardBox(incoming);
    outgoing.hidden = false;
    incoming.classList.add("fold-measure");
    return { win: [window.innerWidth, window.innerHeight], from, to };
  },
  settle: (_w: number, _h: number): Promise<void> => Promise.resolve(),
  /** True when the backend holds a frozen picture over the final resize. */
  end: (): Promise<boolean> => Promise.resolve(false),
  /** Draws the beacon in `win`'s corner; settles when the backend has seen it and dropped its picture. */
  shown: (_win: Box): Promise<void> => Promise.resolve(),
  region: undefined as ((rects: Box[], gapMs: number) => void) | undefined,
};

/** False until the first mode is on screen; the launch shape never animates. */
let ready = false;
let folding = false;
let foldToken = 0;

function applyMode(v: ModeView) {
  document.body.toggleAttribute("data-peek", !!v.peek);
  // A peek pinned mid-unfold keeps its animation.
  if (folding && v.layout === view.layout && v.anchor === view.anchor) {
    view = v;
    return;
  }
  const token = ++foldToken;
  if (folding) {
    // A switch arrived mid-animation: land the one in flight, then start from there.
    folding = false;
    clearFold(shellEl, stripEl);
    setShape(view);
  }
  const switching = ready && (v.layout === "panel") !== (view.layout === "panel") && !reducedMotion();
  if (!switching) return setShape(v);
  fold(v, token).catch((err) => {
    console.error("peepthis: switch animation failed", err);
    if (token !== foldToken) return;
    folding = false;
    clearFold(shellEl, stripEl);
    setShape(v);
    void surface.end();
  });
}

async function fold(v: ModeView, token: number) {
  const toPanel = v.layout === "panel";
  const incoming = toPanel ? shellEl : stripEl;
  folding = true;
  view = v;
  if (!toPanel) stripEl.dataset.layout = v.layout;
  document.body.dataset.anchor = v.anchor;
  incoming.classList.add("fold-measure");
  incoming.hidden = false;
  const land = () => {
    folding = false;
    clearFold(shellEl, stripEl);
    setShape(view);
  };
  // WebView2 goes blank for a few frames when its window resizes. The backend freezes a picture of
  // the card over each resize and drops it when it sees this side's beacon on screen.
  const g = await surface.begin(v);
  if (token !== foldToken) return;
  if (!g) return land();
  await surface.settle(g.win[0], g.win[1]);
  if (token !== foldToken) return;
  const hold = g.covered ? surface.shown(g.from) : undefined;
  await playFold({ shell: shellEl, strip: stripEl, toPanel, g, region: surface.region, hold });
  if (token !== foldToken) return;
  // The last step must be on screen before the backend takes its picture of it.
  await painted();
  if (token !== foldToken) return;
  const covered = await surface.end();
  await surface.settle(g.to.w, g.to.h);
  if (token !== foldToken) return;
  land();
  if (covered) await surface.shown({ x: 0, y: 0, w: g.to.w, h: g.to.h });
}

const root = document.querySelector<HTMLElement>("#sessions")!;
const limitsEl = document.querySelector<HTMLElement>("#limits")!;
const settingsEl = document.querySelector<HTMLElement>("#settings")!;
const settingsBtn = document.querySelector<HTMLButtonElement>("#settings-btn")!;

// peep.js stops its rAF loop when no <peep-bot> is connected, so detaching the list idles rendering while hidden.
const scroller = document.querySelector<HTMLElement>("#scroller")!;
function setPeepsAttached(attached: boolean) {
  if (attached && !root.isConnected) scroller.appendChild(root);
  else if (!attached && root.isConnected) root.remove();
  if (attached && !stripEl.isConnected) document.body.appendChild(stripEl);
  else if (!attached && stripEl.isConnected) stripEl.remove();
}

function showNotice(text: string) {
  const n = document.createElement("div");
  n.className = "notice";
  n.textContent = text;
  root.prepend(n);
}

function mountSettings(api: SettingsApi) {
  const settings = createSettings(settingsEl, { ...api, theme }, () => showSettings(false));
  function showSettings(open: boolean) {
    settingsEl.hidden = !open;
    root.hidden = open;
    settingsBtn.setAttribute("aria-pressed", String(open));
    if (open) void settings.refresh();
  }
  settingsBtn.hidden = false;
  settingsBtn.addEventListener("click", () => showSettings(settingsEl.hidden !== false));
  return {
    show: showSettings,
    refresh: () => settings.refresh(),
    hotkeyStatus: settings.hotkeyStatus,
    toggleHotkeyStatus: settings.toggleHotkeyStatus,
    rulesChanged: settings.rulesChanged,
  };
}

function mountInbox(deps: Pick<InboxDeps, "reveal" | "focusTerminal" | "leave">) {
  return createInbox({
    header: document.querySelector<HTMLElement>("#header")!,
    badge: document.querySelector<HTMLElement>("#pending")!,
    ...deps,
  });
}

/** A clash with another app stays on screen until a free combination is saved. */
function createHotkeyNotice(openSettings: () => void, name: string) {
  const n = document.createElement("div");
  n.className = "notice notice--warn";
  n.hidden = true;
  const text = document.createElement("span");
  const change = document.createElement("button");
  change.type = "button";
  change.className = "card-btn";
  change.textContent = "Change";
  change.addEventListener("click", openSettings);
  n.append(text, change);
  root.prepend(n);
  return (s: HotkeyStatus) => {
    n.hidden = s.registered;
    text.textContent = s.error ?? `The ${name} hotkey ${s.hotkey} is not active.`;
  };
}

function runDemo() {
  let snap = demoSnapshot(Date.now());
  let inbox: ReturnType<typeof mountInbox> | null = null;
  const render = createList(root, demoIdentity, undefined, async (id, decision) => {
    snap = demoDecide(snap, id, decision, Date.now());
    render(snap.sessions);
    inbox?.sync(snap.sessions);
    return true;
  });
  const renderStrip = createStrip(stripEl, stripActions, demoIdentity);
  let trailOn = true;
  let hotkey: HotkeyStatus = { hotkey: "Ctrl+Alt+I", registered: true };
  let toggleHotkey: HotkeyStatus = { hotkey: "Ctrl+Alt+P", registered: true };
  const settings = mountSettings({
    log: async () => [],
    enabled: async () => trailOn,
    setEnabled: async (on) => (trailOn = on),
    hotkey: { get: async () => hotkey, set: async (combo) => (hotkey = { hotkey: combo, registered: true }) },
    toggleHotkey: { get: async () => toggleHotkey, set: async (combo) => (toggleHotkey = { hotkey: combo, registered: true }) },
  });
  inbox = mountInbox({ reveal: () => settings.show(false), focusTerminal: () => {}, leave: () => {} });
  inbox.setHotkey(hotkey.hotkey);
  // No global hotkey in a browser: the same combination works while the page has focus.
  window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.altKey && e.code === "KeyI") {
      e.preventDefault();
      inbox?.open();
    }
  });
  const renderLimits = createLimits(limitsEl);
  if (params.has("digest")) {
    const renderDigest = createDigestCard(
      root,
      { focus: () => {}, openInbox: () => inbox?.open(), dismiss: () => renderDigest(null) },
      () => !!inbox?.active,
    );
    renderDigest(demoDigest(snap, Date.now()));
  }
  render(snap.sessions);
  renderStrip(snap.sessions);
  inbox.sync(snap.sessions);
  renderLimits(snap.rateLimits);
  setInterval(() => {
    snap = demoStep(snap, Date.now());
    render(snap.sessions);
    renderStrip(snap.sessions);
    inbox?.sync(snap.sessions);
    renderLimits(snap.rateLimits);
  }, DEMO_STEP_MS);
}

/** `?empty` shows the house peep; `?empty=banner` adds the first-run banner, `?empty=cycle` brings one session in and out. */
function runEmpty(mode: string) {
  const render = createList(root, demoIdentity);
  const renderStrip = createStrip(stripEl, stripActions, demoIdentity);
  if (mode === "banner") createBanner(root, { install: async () => {} }).render("missing");
  render([]);
  renderStrip([]);
  if (mode !== "cycle") return;
  const one = demoSnapshot(Date.now()).sessions.slice(0, 1);
  let on = false;
  setInterval(() => {
    on = !on;
    render(on ? one : []);
    renderStrip(on ? one : []);
  }, DEMO_STEP_MS);
}

async function runLive() {
  if (!("__TAURI_INTERNALS__" in window)) {
    createList(root)([]);
    createStrip(stripEl, stripActions)([]);
    showNotice("Not running inside Tauri. Add ?demo to the URL for mock sessions.");
    return;
  }
  const [{ invoke }, { listen }] = await Promise.all([import("@tauri-apps/api/core"), import("@tauri-apps/api/event")]);
  let recent: Record<string, TrailEntry[]> = {};
  let sessions: Session[] = [];
  let rulesView: RulesView = EMPTY_RULES;
  const renderList = createList(
    root,
    undefined,
    (id) => invoke<boolean>("focus_session", { sessionId: id }),
    (requestId, decision, reason) => invoke<boolean>("decide_approval", { requestId, decision, reason }),
    (id) => recent[id],
    {
      offersOf: (id) => offersFor(rulesView, id),
      answer: (offerId, answer) => invoke<RulesView>("answer_rule_offer", { offerId, answer }),
      revoke: (ruleId) => invoke<RulesView>("revoke_rule", { ruleId }),
      live: (id) => ruleLive(rulesView, id),
    },
  );
  const renderStrip = createStrip(stripEl, stripActions);
  const settings = mountSettings({
    log: () => invoke<TrailEntry[]>("trail_log"),
    enabled: () => invoke<boolean>("trail_enabled"),
    setEnabled: (on) => invoke<boolean>("set_trail_enabled", { on }),
    hotkey: {
      get: () => invoke<HotkeyStatus>("inbox_hotkey"),
      set: (hotkey) => invoke<HotkeyStatus>("set_inbox_hotkey", { hotkey }),
    },
    toggleHotkey: {
      get: () => invoke<HotkeyStatus>("toggle_hotkey"),
      set: (hotkey) => invoke<HotkeyStatus>("set_toggle_hotkey", { hotkey }),
    },
    rules: {
      view: () => invoke<RulesView>("rules_view"),
      revoke: (ruleId) => invoke<RulesView>("revoke_rule", { ruleId }),
    },
    away: {
      get: () => invoke<number>("away_after"),
      set: (minutes) => invoke<number>("set_away_after", { minutes }),
    },
    summary: {
      get: () => invoke<boolean>("digest_summary"),
      set: (on) => invoke<boolean>("set_digest_summary", { on }),
    },
  });
  await listen("open-settings", () => settings.show(true));
  const applyRules = (v: RulesView) => {
    rulesView = v;
    renderList(sessions);
    settings.rulesChanged(v);
  };
  await listen<RulesView>("rules", (e) => applyRules(e.payload));
  invoke<RulesView>("rules_view").then(applyRules, (err) => console.error("peepthis: rules_view failed", err));
  const inbox = mountInbox({
    reveal: () => settings.show(false),
    focusTerminal: (id) => void invoke<boolean>("focus_session", { sessionId: id }).catch((err) => console.error("peepthis: focus_session failed", err)),
    leave: (restore) => void invoke<boolean>("inbox_return", { restore }).catch((err) => console.error("peepthis: inbox_return failed", err)),
  });
  const hotkeyNotice = createHotkeyNotice(() => settings.show(true), "inbox");
  const applyHotkey = (s: HotkeyStatus) => {
    hotkeyNotice(s);
    inbox.setHotkey(s.registered ? s.hotkey : null);
    settings.hotkeyStatus(s);
  };
  let hotkeyEvent = false;
  await listen<HotkeyStatus>("hotkey-status", (e) => {
    hotkeyEvent = true;
    applyHotkey(e.payload);
  });
  invoke<HotkeyStatus>("inbox_hotkey").then((s) => hotkeyEvent || applyHotkey(s), (err) => console.error("peepthis: inbox_hotkey failed", err));
  const toggleNotice = createHotkeyNotice(() => settings.show(true), "panel/strip");
  const applyToggleHotkey = (s: HotkeyStatus) => {
    toggleNotice(s);
    settings.toggleHotkeyStatus(s);
    collapseBtn.title = s.registered ? `Collapse to strip (${s.hotkey})` : "Collapse to strip";
  };
  let toggleEvent = false;
  await listen<HotkeyStatus>("toggle-hotkey-status", (e) => {
    toggleEvent = true;
    applyToggleHotkey(e.payload);
  });
  invoke<HotkeyStatus>("toggle_hotkey").then((s) => toggleEvent || applyToggleHotkey(s), (err) => console.error("peepthis: toggle_hotkey failed", err));
  await listen("inbox-open", () => inbox.open());
  await listen("inbox-empty", () => inbox.flash());
  const dismissDigest = () => void invoke("dismiss_digest").catch((err) => console.error("peepthis: dismiss_digest failed", err));
  const renderDigest = createDigestCard(
    root,
    {
      focus: (id) => void invoke<boolean>("focus_session", { sessionId: id }).catch((err) => console.error("peepthis: focus_session failed", err)),
      openInbox: () => inbox.open(),
      dismiss: dismissDigest,
    },
    () => inbox.active || settingsEl.hidden === false,
  );
  await listen<Digest | null>("digest", (e) => renderDigest(e.payload));
  invoke<Digest | null>("get_digest").then(renderDigest, (err) => console.error("peepthis: get_digest failed", err));
  const loadRecent = async () => {
    try {
      recent = await invoke<Record<string, TrailEntry[]>>("trail_recent", { perSession: ROW_TRAIL });
      renderList(sessions);
      inbox.sync(sessions);
    } catch (err) {
      console.error("peepthis: trail_recent failed", err);
    }
  };
  await listen<TrailEntry>("trail", () => {
    void loadRecent();
    if (settingsEl.hidden === false) void settings.refresh();
  });
  void loadRecent();
  const renderLimits = createLimits(limitsEl, () =>
    invoke<Snapshot>("get_snapshot").then((s) => s.rateLimits?.at),
  );
  const banner = createBanner(root, { install: () => invoke("install_integration") });
  await listen<TrayResult>("integration-result", (e) => banner.trayResult(e.payload));
  await listen<boolean>("widget-hidden", (e) => setPeepsAttached(!e.payload));
  const render = (snap: Snapshot) => {
    sessions = snap.sessions;
    renderList(snap.sessions);
    renderStrip(snap.sessions);
    flySessions = snap.sessions;
    syncFlyouts();
    inbox.sync(snap.sessions);
    renderLimits(snap.rateLimits);
    banner.render(snap.integration);
  };
  // Subscribe before fetching so no change is lost; a pushed snapshot beats the initial fetch.
  let pushed = false;
  await listen<Snapshot>("sessions", (e) => {
    pushed = true;
    render(e.payload);
  });
  try {
    const snap = await invoke<Snapshot>("get_snapshot");
    if (!pushed) render(snap);
  } catch (err) {
    console.error("peepthis: get_snapshot failed", err);
    if (!pushed) renderList([]);
  }
}

/** Mode and window size belong to the backend: follow its `mode`, report what the current shape needs. */
async function startSurface() {
  if (!("__TAURI_INTERNALS__" in window)) {
    // In a browser `?strip` or `?strip=row` shows the plate; a click opens the panel.
    const strip = params.get("strip");
    if (strip !== null) applyMode({ mode: "strip", layout: strip === "row" ? "row" : "column", anchor: params.get("anchor") ?? "float" });
    else if (params.has("anchor")) applyMode({ ...view, anchor: params.get("anchor")! });
    // No global hotkey in a browser: the toggle works while the page has focus.
    window.addEventListener("keydown", (e) => {
      if (!(e.ctrlKey && e.altKey && e.code === "KeyP")) return;
      e.preventDefault();
      applyMode(view.layout === "panel" ? { ...view, mode: "strip", layout: "column" } : { ...view, mode: "panel", layout: "panel" });
    });
    ready = true;
    return;
  }
  const [{ invoke }, { listen }, { getCurrentWindow }] = await Promise.all([
    import("@tauri-apps/api/core"),
    import("@tauri-apps/api/event"),
    import("@tauri-apps/api/window"),
  ]);
  stripActions.open = () => void invoke("switch_mode", { mode: "panel" }).catch((err) => console.error("peepthis: switch_mode failed", err));
  collapse = () => void invoke("switch_mode", { mode: "strip" }).catch((err) => console.error("peepthis: switch_mode failed", err));
  stripActions.drag =() => void getCurrentWindow().startDragging().catch((err) => console.error("peepthis: drag failed", err));
  // Capture phase, so a press that starts a header drag pins too.
  document.addEventListener(
    "pointerdown",
    (e) => {
      if (view.peek && pinsPeek(e.target as Element)) stripActions.open();
    },
    true,
  );
  let uncovered: (() => void) | null = null;
  await listen("uncovered", () => uncovered?.());
  await listen<{ id: string; since: number }>("flyout-dismiss", (e) => {
    dismissed.set(e.payload.id, e.payload.since);
    syncFlyouts();
  });
  sendFlyouts = (items) => void invoke("set_flyouts", { items }).catch((err) => console.error("peepthis: set_flyouts failed", err));
  syncFlyouts();
  await listen<ModeView>("mode", (e) => applyMode(e.payload));
  applyMode(await invoke<ModeView>("get_mode"));

  const header = document.querySelector<HTMLElement>("#header")!;
  const px = (el: HTMLElement, keys: string[]) => {
    const cs = getComputedStyle(el) as unknown as Record<string, string>;
    return keys.reduce((sum, k) => sum + (parseFloat(cs[k]) || 0), 0);
  };
  // The card's border and the margin that holds its hard shadow sit outside the measured parts.
  const chrome = px(shellEl, ["borderTopWidth", "borderBottomWidth", "marginTop", "marginBottom"]);
  const shadow = px(stripEl, ["marginBottom"]);
  const lengthOf = (layout: ModeView["layout"]) => {
    if (layout === "panel") return [header, root, settingsEl].reduce((sum, el) => sum + el.getBoundingClientRect().height, chrome);
    const r = stripEl.getBoundingClientRect();
    return (layout === "row" ? r.width : r.height) + shadow;
  };
  type FoldView = { win: [number, number]; from: number[]; to: number[]; covered: boolean };
  surface = {
    begin: async (v) => {
      const f = await invoke<FoldView | null>("begin_switch", { height: Math.ceil(lengthOf(v.layout)), layout: v.layout });
      return f && { win: f.win, from: boxOf(f.from), to: boxOf(f.to), covered: f.covered };
    },
    settle: viewportIs,
    shown: (win) =>
      new Promise<void>((resolve) => {
        const el = showBeacon(win);
        const done = () => {
          clearTimeout(timer);
          uncovered = null;
          el.remove();
          resolve();
        };
        // The backend gives up on its own after 450ms.
        const timer = setTimeout(done, 600);
        uncovered = done;
      }),
    end: () => invoke<boolean>("end_switch"),
    region: (rects, gapMs) =>
      void invoke("play_region", { rects: rects.map((b) => [b.x, b.y, b.w, b.h]), gapMs: Math.round(gapMs) }).catch((err) =>
        console.error("peepthis: play_region failed", err),
      ),
  };
  remeasure = watchContent(
    [header, root, settingsEl, stripEl],
    () => (folding || view.mode === "hidden" ? null : { length: lengthOf(view.layout), tag: view.layout }),
    (height, layout) => {
      invoke("fit_window", { height, layout }).catch((err) => console.error("peepthis: fit_window failed", err));
    },
  );
  remeasure();
  ready = true;
}

const params = new URLSearchParams(location.search);
startSurface().catch((err) => console.error("peepthis: surface failed", err));

if (params.has("accessories")) renderAccessoryCheck(root);
else if (params.has("demo")) runDemo();
else if (params.has("empty")) runEmpty(params.get("empty")!);
else runLive().catch((err) => {
  console.error("peepthis: startup failed", err);
  showNotice("Could not connect to the Peep This backend.");
});
