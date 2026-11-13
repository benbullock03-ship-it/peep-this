import "./vendor/peep.js";
import { demoIdentity, demoSnapshot, demoStep } from "./demo";
import type { Snapshot } from "./model";
import { createList } from "./rows";
import { watchContentHeight } from "./windowFit";

const DEMO_STEP_MS = 3200;

const root = document.querySelector<HTMLElement>("#sessions")!;

function showNotice(text: string) {
  const n = document.createElement("div");
  n.className = "notice";
  n.textContent = text;
  root.prepend(n);
}

function runDemo() {
  const render = createList(root, demoIdentity);
  let snap = demoSnapshot(Date.now());
  render(snap.sessions);
  setInterval(() => {
    snap = demoStep(snap, Date.now());
    render(snap.sessions);
  }, DEMO_STEP_MS);
}

async function runLive() {
  if (!("__TAURI_INTERNALS__" in window)) {
    createList(root)([]);
    showNotice("Not running inside Tauri. Add ?demo to the URL for mock sessions.");
    return;
  }
  const [{ invoke }, { listen }] = await Promise.all([import("@tauri-apps/api/core"), import("@tauri-apps/api/event")]);
  const render = createList(root, undefined, (id) => invoke<boolean>("focus_session", { sessionId: id }));
  // Subscribe before fetching so no change is lost; a pushed snapshot beats the initial fetch.
  let pushed = false;
  await listen<Snapshot>("sessions", (e) => {
    pushed = true;
    render(e.payload.sessions);
  });
  try {
    const snap = await invoke<Snapshot>("get_snapshot");
    if (!pushed) render(snap.sessions);
  } catch (err) {
    console.error("agentview: get_snapshot failed", err);
    if (!pushed) render([]);
  }
}

async function startWindowFit() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  const { invoke } = await import("@tauri-apps/api/core");
  const header = document.querySelector<HTMLElement>("#header")!;
  watchContentHeight([header, root], (height) => {
    invoke("fit_window", { height }).catch((err) => console.error("agentview: fit_window failed", err));
  });
}

startWindowFit().catch((err) => console.error("agentview: window fit failed", err));

if (new URLSearchParams(location.search).has("demo")) runDemo();
else runLive().catch((err) => {
  console.error("agentview: startup failed", err);
  showNotice("Could not connect to the AgentView backend.");
});
