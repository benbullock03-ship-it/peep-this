import { AWAY_CHOICES } from "./digest";
import { comboFromKey } from "./inbox";
import type { RulesView, TrailEntry, TrailOutcome } from "./model";
import { usesText } from "./rules";
import { THEME_PREFS, type ThemePref } from "./theme";
import { entryTitle, filterTrail, formatWhen, LOG_MAX, markOf, OUTCOMES, sessionOptions, shownTarget, type TrailFilter } from "./trail";

/** hotkey.rs `Status`. */
export type HotkeyStatus = { hotkey: string; registered: boolean; error?: string };

type HotkeyApi = { get: () => Promise<HotkeyStatus>; set: (combo: string) => Promise<HotkeyStatus> };

export type SettingsApi = {
  log: () => Promise<TrailEntry[]>;
  enabled: () => Promise<boolean>;
  setEnabled: (on: boolean) => Promise<boolean>;
  hotkey?: HotkeyApi;
  toggleHotkey?: HotkeyApi;
  theme?: { get: () => ThemePref; set: (pref: ThemePref) => void };
  rules?: { view: () => Promise<RulesView>; revoke: (ruleId: string) => Promise<RulesView> };
  away?: { get: () => Promise<number>; set: (minutes: number) => Promise<number> };
  summary?: { get: () => Promise<boolean>; set: (on: boolean) => Promise<boolean> };
};

const INBOX_FIELD = {
  title: "Inbox hotkey",
  note: "Brings the oldest pending approval into focus. Y allow, N deny, R reason, Tab next, Enter terminal, Esc back.",
};
const TOGGLE_FIELD = {
  title: "Panel/strip hotkey",
  note: "Opens the panel from the strip, or folds it back, without leaving the app you are in.",
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

  const hotkey = api.hotkey ? createHotkeyField(host, api.hotkey, INBOX_FIELD) : null;
  const toggleHotkey = api.toggleHotkey ? createHotkeyField(host, api.toggleHotkey, TOGGLE_FIELD) : null;
  const themeSelect = api.theme ? createThemeField(host, api.theme) : null;
  const away = api.away ? createAwayField(host, api.away) : null;
  const summary = api.summary ? createSummaryField(host, api.summary) : null;
  const rules = api.rules ? createRulesField(host, api.rules) : null;

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
    if (themeSelect && api.theme) themeSelect.value = api.theme.get();
    await Promise.all([hotkey?.refresh(), toggleHotkey?.refresh(), rules?.refresh(), away?.refresh(), summary?.refresh()]);
  }

  return {
    refresh,
    rulesChanged: (v: RulesView) => rules?.show(v),
    hotkeyStatus: (s: HotkeyStatus) => hotkey?.show(s),
    toggleHotkeyStatus: (s: HotkeyStatus) => toggleHotkey?.show(s),
  };
}

/** AV-31: confirmed auto-allow rules with their use counts; Delete revokes. */
function createRulesField(host: HTMLElement, api: NonNullable<SettingsApi["rules"]>) {
  el("h3", "settings-sub", host, "Auto-allow rules");
  el("p", "settings-note", host, "Offered after three identical allows in one project. Never used for destructive requests.");
  const msg = el("p", "settings-error", host);
  msg.hidden = true;
  const list = el("div", "rule-list", host);

  function show(v: RulesView) {
    msg.hidden = !v.error;
    msg.textContent = v.error ? `${v.error}. No rule applies until it is fixed.` : "";
    list.replaceChildren();
    if (!v.error && v.rules.length === 0) el("div", "log-count", list, "No rules yet.");
    for (const r of v.rules) {
      const item = el("div", "rule-item", list);
      item.dataset.id = r.id;
      const what = el("span", "log-what", item);
      el("span", "trail-tool", what, r.tool);
      el("code", "rule-pattern", what, r.pattern).title = r.pattern;
      const del = el("button", "card-btn rule-delete", item, "Delete");
      del.type = "button";
      del.addEventListener("click", async () => {
        del.disabled = true;
        try {
          show(await api.revoke(r.id));
        } catch (e) {
          msg.textContent = `Could not delete: ${e}`;
          msg.hidden = false;
          del.disabled = false;
        }
      });
      el("span", "rule-meta", item, `${usesText(r)} · ${r.cwd}`).title = r.cwd;
    }
  }

  return {
    show,
    async refresh() {
      try {
        show(await api.view());
      } catch (e) {
        msg.textContent = `Could not read the rules: ${e}`;
        msg.hidden = false;
      }
    },
  };
}

const THEME_LABELS: Record<ThemePref, string> = { system: "Match Windows", dark: "Dark", light: "Light" };

function createThemeField(host: HTMLElement, api: NonNullable<SettingsApi["theme"]>) {
  el("h3", "settings-sub", host, "Theme");
  const select = el("select", "log-select theme-select", host);
  select.setAttribute("aria-label", "Theme");
  for (const p of THEME_PREFS) el("option", "", select, THEME_LABELS[p]).value = p;
  select.value = api.get();
  select.addEventListener("change", () => api.set(select.value as ThemePref));
  return select;
}

/** AV-32: how long without input counts as away; a locked screen always does. */
function createAwayField(host: HTMLElement, api: NonNullable<SettingsApi["away"]>) {
  el("h3", "settings-sub", host, "Away after");
  const select = el("select", "log-select away-select", host);
  select.setAttribute("aria-label", "Away after");
  for (const m of AWAY_CHOICES) el("option", "", select, `${m} min without input`).value = String(m);
  el("p", "settings-note", host, "Locking the screen also counts. On return, one card says what happened meanwhile.");
  const msg = el("p", "settings-error", host);
  msg.hidden = true;
  select.addEventListener("change", async () => {
    select.disabled = true;
    try {
      select.value = String(await api.set(Number(select.value)));
      msg.hidden = true;
    } catch (e) {
      msg.textContent = `Could not save: ${e}`;
      msg.hidden = false;
    } finally {
      select.disabled = false;
    }
  });
  return {
    async refresh() {
      try {
        select.value = String(await api.get());
      } catch (e) {
        msg.textContent = `Could not read the setting: ${e}`;
        msg.hidden = false;
      }
    },
  };
}

/** AV-33: opt-in model sentence per digest entry. */
function createSummaryField(host: HTMLElement, api: NonNullable<SettingsApi["summary"]>) {
  const row = el("label", "settings-toggle", host);
  const box = el("input", "", row);
  box.type = "checkbox";
  el("span", "", row, "Summarise the away card with Haiku");
  el(
    "p",
    "settings-note",
    host,
    "Off by default. Each return runs one claude -p call: it spends your rate limit, needs the network, and sends each session's name, changed file names, counts and last line to the model. The facts card shows first either way.",
  );
  const msg = el("p", "settings-error", host);
  msg.hidden = true;
  box.addEventListener("change", async () => {
    box.disabled = true;
    try {
      box.checked = await api.set(box.checked);
      msg.hidden = true;
    } catch (e) {
      box.checked = !box.checked;
      msg.textContent = `Could not save: ${e}`;
      msg.hidden = false;
    } finally {
      box.disabled = false;
    }
  });
  return {
    async refresh() {
      try {
        box.checked = await api.get();
      } catch (e) {
        msg.textContent = `Could not read the setting: ${e}`;
        msg.hidden = false;
      }
    },
  };
}

/** A global hotkey: press a combination in the field, then Save (Enter). Esc puts the live one back. */
function createHotkeyField(host: HTMLElement, api: HotkeyApi, text: { title: string; note: string }) {
  el("h3", "settings-sub", host, text.title);
  const row = el("div", "hotkey-row", host);
  const input = el("input", "hotkey-input", row);
  input.readOnly = true;
  input.setAttribute("aria-label", text.title);
  input.placeholder = "Press a combination";
  const save = el("button", "card-btn hotkey-save", row, "Save");
  save.type = "button";
  el("p", "settings-note", host, text.note);
  const msg = el("p", "settings-error", host);
  msg.hidden = true;

  let live: HotkeyStatus | null = null;

  function show(s: HotkeyStatus) {
    live = s;
    input.value = s.hotkey;
    msg.hidden = !s.error;
    msg.textContent = s.error ?? "";
  }

  async function commit() {
    if (!input.value || (live?.registered && input.value === live.hotkey)) return;
    save.disabled = true;
    try {
      show(await api.set(input.value));
    } catch (e) {
      msg.textContent = String(e);
      msg.hidden = false;
    } finally {
      save.disabled = false;
    }
  }

  input.addEventListener("keydown", (e) => {
    const bare = !e.ctrlKey && !e.altKey && !e.metaKey && !e.shiftKey;
    if (e.key === "Tab" && bare) return;
    e.preventDefault();
    if (e.key === "Enter" && bare) void commit();
    else if (e.key === "Escape" && bare) {
      if (live) show(live);
    } else {
      const combo = comboFromKey(e);
      if (combo) input.value = combo;
    }
  });
  save.addEventListener("click", () => void commit());

  return {
    show,
    async refresh() {
      try {
        show(await api.get());
      } catch (e) {
        msg.textContent = `Could not read the hotkey: ${e}`;
        msg.hidden = false;
      }
    },
  };
}
