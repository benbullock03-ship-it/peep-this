import "./vendor/peep.js";
import { renderAccessoryCheck } from "./accessories";
import { createBanner, type TrayResult } from "./banner";
import { demoIdentity, demoSnapshot, demoStep } from "./demo";
import { createLimits } from "./limits";
import type { Snapshot } from "./model";
import { createList } from "./rows";
import { watchContentHeight } from "./windowFit";

const DEMO_STEP_MS = 3200;

const root = document.querySelector<HTMLElement>("#sessions")!;
const limitsEl = document.querySelector<HTMLElement>("#limits")!;

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

function runDemo() {
  const render = createList(root, demoIdentity);
  const renderLimits = createLimits(limitsEl);
  let snap = demoSnapshot(Date.now());
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
  const renderList = createList(root, undefined, (id) => invoke<boolean>("focus_session", { sessionId: id }));
  const renderLimits = createLimits(limitsEl, () =>
    invoke<Snapshot>("get_snapshot").then((s) => s.rateLimits?.at),
  );
  const banner = createBanner(root, { install: () => invoke("install_integration") });
  await listen<TrayResult>("integration-result", (e) => banner.trayResult(e.payload));
  await listen<boolean>("widget-hidden", (e) => setPeepsAttached(!e.payload));
  const renderApprovals = import.meta.env.DEV
    ? (await import("./devApprovals")).createDevApprovals(document.body, (requestId, decision, reason) =>
        invoke<boolean>("decide_approval", { requestId, decision, reason }),
      )
    : undefined;
  const render = (snap: Snapshot) => {
    renderApprovals?.(snap.sessions);
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
  watchContentHeight([header, root], (height) => {
    if (!root.isConnected) return;
    invoke("fit_window", { height }).catch((err) => console.error("peepthis: fit_window failed", err));
  });
}

startWindowFit().catch((err) => console.error("peepthis: window fit failed", err));

const params = new URLSearchParams(location.search);
if (params.has("accessories")) renderAccessoryCheck(root);
else if (params.has("demo")) runDemo();
else runLive().catch((err) => {
  console.error("peepthis: startup failed", err);
  showNotice("Could not connect to the Peep This backend.");
});
