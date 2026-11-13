import type { Snapshot } from "./model";

export type Integration = Snapshot["integration"];
export type BannerMode = "install" | "update" | "disconnect" | null;

const DISMISS_KEY = "peepthis.integrationBanner.dismissed";
const SETTLE_MS = 1500;

/** `dismissed` is the integration state the user last closed the banner on; a new state shows it again. */
export function bannerMode(integration: Integration, dismissed: string | null, disconnectAsked: boolean): BannerMode {
  if (disconnectAsked) return integration === "missing" ? "install" : integration === "installed" ? "disconnect" : "update";
  if (integration === dismissed) return null;
  return integration === "missing" ? "install" : integration === "outdated" ? "update" : null;
}

const COPY: Record<Exclude<BannerMode, null>, { text: string; action: string; busy: string }> = {
  install: { text: "Connect Peep This to Claude Code", action: "Install", busy: "Installing…" },
  update: { text: "Peep This's Claude Code connection is out of date", action: "Update", busy: "Updating…" },
  disconnect: { text: "Peep This is connected to Claude Code", action: "Disconnect", busy: "Disconnecting…" },
};

function readDismissed(): string | null {
  try {
    return localStorage.getItem(DISMISS_KEY);
  } catch {
    return null;
  }
}

function writeDismissed(value: Integration) {
  try {
    localStorage.setItem(DISMISS_KEY, value);
  } catch {
    // Dismissal then lasts only for this session.
  }
}

export type BannerActions = { install: () => Promise<unknown>; uninstall: () => Promise<unknown> };

/** Prepends the integration banner to `parent`. `toggleDisconnect` opens it on demand (header right-click). */
export function createBanner(parent: HTMLElement, actions: BannerActions) {
  const el = document.createElement("div");
  el.className = "banner";
  el.hidden = true;
  const text = document.createElement("span");
  text.className = "banner-text";
  const action = document.createElement("button");
  action.className = "banner-action";
  const close = document.createElement("button");
  close.className = "banner-close";
  close.textContent = "×";
  close.title = "Dismiss";
  close.setAttribute("aria-label", "Dismiss");
  const error = document.createElement("div");
  error.className = "banner-error";
  error.hidden = true;
  el.append(text, action, close, error);
  parent.prepend(el);

  let integration: Integration | null = null;
  let dismissed = readDismissed();
  let asked = false;
  let busy = false;
  let mode: BannerMode = null;
  let settleTimer = 0;

  const settle = () => {
    window.clearTimeout(settleTimer);
    settleTimer = 0;
    busy = false;
    update();
  };

  const update = () => {
    if (busy || integration === null) return;
    const next = bannerMode(integration, dismissed, asked);
    if (next !== mode) error.hidden = true;
    mode = next;
    el.hidden = mode === null;
    if (mode === null) return;
    text.textContent = COPY[mode].text;
    action.textContent = COPY[mode].action;
    action.disabled = false;
  };

  action.addEventListener("click", async () => {
    if (!mode || busy) return;
    const run = mode === "disconnect" ? actions.uninstall : actions.install;
    const before = integration;
    busy = true;
    action.disabled = true;
    action.textContent = COPY[mode].busy;
    error.hidden = true;
    try {
      await run();
      if (mode === "disconnect") {
        dismissed = "missing";
        writeDismissed("missing");
      }
      asked = false;
      // Stay busy until the snapshot with the new status lands, so the old button does not flash back.
      if (integration !== before) settle();
      else settleTimer = window.setTimeout(settle, SETTLE_MS);
    } catch (err) {
      error.textContent = String(err);
      error.hidden = false;
      settle();
    }
  });

  close.addEventListener("click", () => {
    if (integration && !asked) {
      dismissed = integration;
      writeDismissed(integration);
    }
    asked = false;
    update();
  });

  return {
    render(next: Integration) {
      const changed = next !== integration;
      integration = next;
      if (busy && changed && settleTimer) settle();
      else update();
    },
    toggleDisconnect() {
      asked = !asked;
      update();
    },
  };
}
