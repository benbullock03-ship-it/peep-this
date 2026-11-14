import "./vendor/peep.js";
import { renderAccessoryCheck } from "./accessories";
import { createBanner, type TrayResult } from "./banner";
import { demoDecide, demoIdentity, demoSnapshot, demoStep } from "./demo";
import { createLimits } from "./limits";
import type { Session, Snapshot, TrailEntry } from "./model";
import { createList } from "./rows";
import { createSettings, type SettingsApi } from "./settings";
import { ROW_TRAIL } from "./trail";
import { watchContentHeight } from "./windowFit";

const DEMO_STEP_MS = 3200;

const root = document.querySelector<HTMLElement>("#sessions")!;
const limitsEl = document.querySelector<HTMLElement>("#limits")!;
const settingsEl = document.querySelector<HTMLElement>("#settings")!;
const settingsBtn = document.querySelector<HTMLButtonElement>("#settings-btn")!;

// peep.js stops its rAF loop when no <peep-bot> is connected, so detaching the list idles rendering while hidden.
const scroller = document.querySelector<HTMLElement>("#scroller")!;
function setPeepsAttached(attached: boolean) {
  if (attached && !root.isConnected) scroller.appendChild(root);
  else if (!attached && root.isConnected) root.remove();
}

function showNotice(text: string) {
  const n = document.createElement("div");
  n.className = "notice";
  n.textContent = text;
  root.prepend(n);
}

function mountSettings(api: SettingsApi) {
  const settings = createSettings(settingsEl, api, () => showSettings(false));
  function showSettings(open: boolean) {
    settingsEl.hidden = !open;
    root.hidden = open;
    settingsBtn.setAttribute("aria-pressed", String(open));
    if (open) void settings.refresh();
  }
  settingsBtn.hidden = false;
  settingsBtn.addEventListener("click", () => showSettings(settingsEl.hidden !== false));
  return { show: showSettings, refresh: () => settings.refresh() };
}

function runDemo() {
  let snap = demoSnapshot(Date.now());
  const render = createList(root, demoIdentity, undefined, async (id, decision) => {
    snap = demoDecide(snap, id, decision, Date.now());
    render(snap.sessions);
    return true;
  });
  let trailOn = true;
  mountSettings({ log: async () => [], enabled: async () => trailOn, setEnabled: async (on) => (trailOn = on) });
  const renderLimits = createLimits(limitsEl);
  render(snap.sessions);
  renderLimits(snap.rateLimits);
  setInterval(() => {
    snap = demoStep(snap, Date.now());
    render(snap.sessions);
    renderLimits(snap.rateLimits);
  }, DEMO_STEP_MS);
}

async function runLive() {
  if (!("__TAURI_INTERNALS__" in window)) {
    createList(root)([]);
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
  const settings = mountSettings({
    log: () => invoke<TrailEntry[]>("trail_log"),
    enabled: () => invoke<boolean>("trail_enabled"),
    setEnabled: (on) => invoke<boolean>("set_trail_enabled", { on }),
  });
  await listen("open-settings", () => settings.show(true));
  const loadRecent = async () => {
    try {
      recent = await invoke<Record<string, TrailEntry[]>>("trail_recent", { perSession: ROW_TRAIL });
      renderList(sessions);
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

async function startWindowFit() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  const { invoke } = await import("@tauri-apps/api/core");
  const header = document.querySelector<HTMLElement>("#header")!;
  const shell = getComputedStyle(document.querySelector<HTMLElement>("#shell")!);
  // The card's border and the margin that holds its hard shadow sit outside the measured parts.
  const chrome = ["borderTopWidth", "borderBottomWidth", "marginTop", "marginBottom"].reduce(
    (sum, k) => sum + (parseFloat(shell[k as keyof CSSStyleDeclaration] as string) || 0),
    0,
  );
  watchContentHeight(
    [header, root, settingsEl],
    (height) => {
      if (!root.isConnected) return;
      invoke("fit_window", { height }).catch((err) => console.error("peepthis: fit_window failed", err));
    },
    chrome,
  );
}

startWindowFit().catch((err) => console.error("peepthis: window fit failed", err));

const params = new URLSearchParams(location.search);
if (params.has("accessories")) renderAccessoryCheck(root);
else if (params.has("demo")) runDemo();
else runLive().catch((err) => {
  console.error("peepthis: startup failed", err);
  showNotice("Could not connect to the Peep This backend.");
});
