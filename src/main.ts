import "./vendor/peep.js";
import { renderAccessoryCheck } from "./accessories";
import { createBanner, type TrayResult } from "./banner";
import { demoDecide, demoIdentity, demoSnapshot, demoStep } from "./demo";
import { createInbox, type InboxDeps } from "./inboxView";
import { createLimits } from "./limits";
import type { Session, Snapshot, TrailEntry } from "./model";
import { createList } from "./rows";
import { createSettings, type HotkeyStatus, type SettingsApi } from "./settings";
import { createStrip, type StripActions } from "./stripPlate";
import { initTheme } from "./theme";
import { ROW_TRAIL } from "./trail";
import { watchContent } from "./windowFit";

const DEMO_STEP_MS = 3200;
const theme = initTheme();

/** Mirrors `ModeView` in src-tauri/src/window.rs. `layout` is the visible shape even while hidden. */
type ModeView = { mode: "panel" | "strip" | "hidden"; layout: "panel" | "column" | "row"; anchor: string };

const shellEl = document.querySelector<HTMLElement>("#shell")!;
const stripEl = document.querySelector<HTMLElement>("#strip")!;
let view: ModeView = { mode: "panel", layout: "panel", anchor: "float" };
let remeasure = () => {};
const stripActions: StripActions = { open: () => applyMode({ ...view, mode: "panel", layout: "panel" }) };

function applyMode(v: ModeView) {
  view = v;
  shellEl.hidden = v.layout !== "panel";
  stripEl.hidden = v.layout === "panel";
  stripEl.dataset.layout = v.layout;
  // styles.css pins the card to the anchored side, so an unfold grows away from it.
  document.body.dataset.anchor = v.anchor;
  remeasure();
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
  return { show: showSettings, refresh: () => settings.refresh(), hotkeyStatus: settings.hotkeyStatus };
}

function mountInbox(deps: Pick<InboxDeps, "reveal" | "focusTerminal" | "leave">) {
  return createInbox({
    header: document.querySelector<HTMLElement>("#header")!,
    badge: document.querySelector<HTMLElement>("#pending")!,
    ...deps,
  });
}

/** A clash with another app stays on screen until a free combination is saved. */
function createHotkeyNotice(openSettings: () => void) {
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
    text.textContent = s.error ?? `The inbox hotkey ${s.hotkey} is not active.`;
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
  const settings = mountSettings({
    log: async () => [],
    enabled: async () => trailOn,
    setEnabled: async (on) => (trailOn = on),
    hotkey: { get: async () => hotkey, set: async (combo) => (hotkey = { hotkey: combo, registered: true }) },
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
  const renderList = createList(
    root,
    undefined,
    (id) => invoke<boolean>("focus_session", { sessionId: id }),
    (requestId, decision, reason) => invoke<boolean>("decide_approval", { requestId, decision, reason }),
    (id) => recent[id],
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
  });
  await listen("open-settings", () => settings.show(true));
  const inbox = mountInbox({
    reveal: () => settings.show(false),
    focusTerminal: (id) => void invoke<boolean>("focus_session", { sessionId: id }).catch((err) => console.error("peepthis: focus_session failed", err)),
    leave: (restore) => void invoke<boolean>("inbox_return", { restore }).catch((err) => console.error("peepthis: inbox_return failed", err)),
  });
  const hotkeyNotice = createHotkeyNotice(() => settings.show(true));
  const applyHotkey = (s: HotkeyStatus) => {
    hotkeyNotice(s);
    inbox.setHotkey(s.registered ? s.hotkey : null);
    settings.hotkeyStatus(s);
  };
  await listen<HotkeyStatus>("hotkey-status", (e) => applyHotkey(e.payload));
  invoke<HotkeyStatus>("inbox_hotkey").then(applyHotkey, (err) => console.error("peepthis: inbox_hotkey failed", err));
  await listen("inbox-open", () => inbox.open());
  await listen("inbox-empty", () => inbox.flash());
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
    return;
  }
  const [{ invoke }, { listen }, { getCurrentWindow }] = await Promise.all([
    import("@tauri-apps/api/core"),
    import("@tauri-apps/api/event"),
    import("@tauri-apps/api/window"),
  ]);
  stripActions.open = () => void invoke("switch_mode", { mode: "panel" }).catch((err) => console.error("peepthis: switch_mode failed", err));
  stripActions.drag = () => void getCurrentWindow().startDragging().catch((err) => console.error("peepthis: drag failed", err));
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
  remeasure = watchContent(
    [header, root, settingsEl, stripEl],
    () => {
      if (view.mode === "hidden") return null;
      if (view.layout === "panel") {
        const height = [header, root, settingsEl].reduce((sum, el) => sum + el.getBoundingClientRect().height, chrome);
        return { length: height, tag: "panel" };
      }
      const r = stripEl.getBoundingClientRect();
      return { length: (view.layout === "row" ? r.width : r.height) + shadow, tag: view.layout };
    },
    (height, layout) => {
      invoke("fit_window", { height, layout }).catch((err) => console.error("peepthis: fit_window failed", err));
    },
  );
  remeasure();
}

const params = new URLSearchParams(location.search);
startSurface().catch((err) => console.error("peepthis: surface failed", err));

if (params.has("accessories")) renderAccessoryCheck(root);
else if (params.has("demo")) runDemo();
else runLive().catch((err) => {
  console.error("peepthis: startup failed", err);
  showNotice("Could not connect to the Peep This backend.");
});
