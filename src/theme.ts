export type ThemePref = "system" | "dark" | "light";
export type Theme = "dark" | "light";

export const THEME_PREFS: readonly ThemePref[] = ["system", "dark", "light"];
const KEY = "peep.theme";

export function resolveTheme(pref: ThemePref, osDark: boolean): Theme {
  return pref === "system" ? (osDark ? "dark" : "light") : pref;
}

export function parsePref(raw: string | null | undefined): ThemePref {
  return THEME_PREFS.includes(raw as ThemePref) ? (raw as ThemePref) : "system";
}

function readPref(): ThemePref {
  try {
    return parsePref(localStorage.getItem(KEY));
  } catch {
    return "system";
  }
}

/** Sets `data-theme` on <html> from the stored preference and follows the OS live while it is "system". */
export function initTheme(root: HTMLElement = document.documentElement) {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  let pref = readPref();
  const apply = () => {
    root.dataset.theme = resolveTheme(pref, mq.matches);
  };
  mq.addEventListener("change", apply);
  apply();
  return {
    get: () => pref,
    set(next: ThemePref) {
      pref = next;
      try {
        localStorage.setItem(KEY, next);
      } catch {
        // Not persisted; still applies for this run.
      }
      apply();
    },
  };
}
