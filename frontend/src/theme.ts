// Theme preference: clair / sombre / automatique (follows the system).
// The resolved theme is always written to <html data-theme="light|dark">, so
// the stylesheet only needs a light default plus a [data-theme="dark"] block.

export type ThemePref = "auto" | "light" | "dark";

const KEY = "theme";
const DARK_BG = "#0f1115";
const LIGHT_BG = "#f7f8fb";

export function getThemePref(): ThemePref {
  try {
    const v = localStorage.getItem(KEY);
    if (v === "light" || v === "dark" || v === "auto") return v;
  } catch {
    // storage unavailable (private mode)
  }
  return "auto";
}

function systemPrefersDark(): boolean {
  return typeof window !== "undefined" && window.matchMedia
    ? window.matchMedia("(prefers-color-scheme: dark)").matches
    : false;
}

function apply(pref: ThemePref): void {
  const dark = pref === "dark" || (pref === "auto" && systemPrefersDark());
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  // Keep the browser/PWA chrome (status bar, address bar) in step.
  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) meta.setAttribute("content", dark ? DARK_BG : LIGHT_BG);
}

/** Persist and apply a new preference. */
export function setThemePref(pref: ThemePref): void {
  try {
    localStorage.setItem(KEY, pref);
  } catch {
    // still applies for this session
  }
  apply(pref);
}

/** Apply the stored preference and keep "auto" in sync with the system. */
export function initTheme(): void {
  apply(getThemePref());
  if (typeof window === "undefined" || !window.matchMedia) return;
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const onChange = () => {
    if (getThemePref() === "auto") apply("auto");
  };
  if (mq.addEventListener) mq.addEventListener("change", onChange);
  else mq.addListener(onChange); // older Safari
}
