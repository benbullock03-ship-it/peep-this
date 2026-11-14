import type { TrailEntry, TrailOutcome } from "./model";
import { entryTitle, filterTrail, formatWhen, LOG_MAX, markOf, OUTCOMES, sessionOptions, shownTarget, type TrailFilter } from "./trail";

export type SettingsApi = {
  log: () => Promise<TrailEntry[]>;
  enabled: () => Promise<boolean>;
  setEnabled: (on: boolean) => Promise<boolean>;
};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, parent?: HTMLElement, text?: string) {
  const e = document.createElement(tag);
  e.className = cls;
  if (text !== undefined) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

/** The settings view: the trail switch and the full, filterable decision log. */
export function createSettings(host: HTMLElement, api: SettingsApi, onClose: () => void) {
  host.replaceChildren();
  const head = el("div", "settings-head", host);
  el("h2", "settings-title", head, "Settings");
  const close = el("button", "settings-close", head, "×");
  close.type = "button";
  close.title = "Back to sessions";
  close.setAttribute("aria-label", "Back to sessions");
  close.addEventListener("click", onClose);

  const toggleRow = el("label", "settings-toggle", host);
  const toggle = el("input", "", toggleRow);
  toggle.type = "checkbox";
  el("span", "", toggleRow, "Record approval decisions");
  el("p", "settings-note", host, "Keeps the command or path, never file contents. Entries older than 90 days are removed.");
  const err = el("p", "settings-error", host);
  err.hidden = true;

  el("h3", "settings-sub", host, "Decision log");
  const filters = el("div", "log-filters", host);
  const text = el("input", "log-search", filters);
  text.type = "search";
  text.placeholder = "Search tool, command, path, reason";
  const session = el("select", "log-select", filters);
  session.setAttribute("aria-label", "Session");
  const outcome = el("select", "log-select", filters);
  outcome.setAttribute("aria-label", "Outcome");
  for (const [v, label] of [["", "All outcomes"], ...OUTCOMES.map((o) => [o, o] as const)]) {
    const opt = el("option", "", outcome, label);
    opt.value = v;
  }
  const count = el("div", "log-count", host);
  const list = el("div", "log-list", host);

  let entries: TrailEntry[] = [];

  function renderSessions() {
    const keep = session.value;
    session.replaceChildren();
    el("option", "", session, "All sessions").value = "";
    for (const s of sessionOptions(entries)) el("option", "", session, s.name).value = s.id;
    session.value = [...session.options].some((o) => o.value === keep) ? keep : "";
  }

  function renderList() {
    const f: TrailFilter = { session: session.value, outcome: outcome.value as TrailOutcome | "", text: text.value };
    const hits = filterTrail(entries, f);
    const shown = hits.slice(0, LOG_MAX);
    count.textContent =
      entries.length === 0
        ? "No decisions recorded yet."
        : hits.length > shown.length
          ? `Showing ${shown.length} of ${hits.length} matches`
          : `${hits.length} of ${entries.length} decisions`;
    const now = Date.now();
    list.replaceChildren();
    for (const e of shown) {
      const item = el("div", "log-item", list);
      item.dataset.outcome = e.outcome;
      item.title = `${e.session}\n${entryTitle(e, now)}`;
      el("span", "trail-mark", item, markOf(e.outcome));
      el("span", "log-when", item, formatWhen(e.t, now));
      el("span", "log-session", item, e.session);
      const what = el("span", "log-what", item);
      el("span", "trail-tool", what, e.tool);
      el("span", "trail-target", what, shownTarget(e));
      if (e.reason) el("span", "log-reason", item, e.reason);
    }
  }

  text.addEventListener("input", renderList);
  session.addEventListener("change", renderList);
  outcome.addEventListener("change", renderList);
  toggle.addEventListener("change", async () => {
    const want = toggle.checked;
    toggle.disabled = true;
    try {
      toggle.checked = await api.setEnabled(want);
      err.hidden = true;
    } catch (e) {
      toggle.checked = !want;
      err.textContent = `Could not save: ${e}`;
      err.hidden = false;
    } finally {
      toggle.disabled = false;
    }
  });

  async function refresh() {
    try {
      const [log, on] = await Promise.all([api.log(), api.enabled()]);
      entries = log;
      toggle.checked = on;
      err.hidden = true;
    } catch (e) {
      err.textContent = `Could not read the log: ${e}`;
      err.hidden = false;
    }
    renderSessions();
    renderList();
  }

  return { refresh };
}
