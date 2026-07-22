import * as log from "../lib/log";
import * as storage from "../lib/storage";

// The header's light/dark toggle. The `.dark` class on <html> is set pre-paint by Base.astro's
// inline bootstrap (a stored choice wins, else the OS preference); this island lets the reader flip
// it and persists the result under THEME_KEY. The Monaco editors observe <html>.dark through a
// MutationObserver, so they recolor on their own — nothing here reaches into them.

/** Is the dark theme currently on? */
function isDark(): boolean {
  return document.documentElement.classList.contains("dark");
}

/** Keep the button's accessible state in step with the theme. The sun/moon glyph swap is pure CSS
 *  (`.dark .header__icon-btn__*` in shell.css); the label announces the action, not the state. */
function reflect(btn: Element, dark: boolean): void {
  btn.setAttribute("aria-pressed", String(dark));
  btn.setAttribute("aria-label", dark ? "Switch to light theme" : "Switch to dark theme");
}

function init(): void {
  const btn = document.querySelector("[data-theme-toggle]");
  if (!btn) return;
  reflect(btn, isDark());
  btn.addEventListener("click", () => {
    const dark = !isDark();
    document.documentElement.classList.toggle("dark", dark);
    storage.set(storage.THEME_KEY, dark ? "dark" : "light");
    reflect(btn, dark);
    log.info(`theme: switched to ${dark ? "dark" : "light"}`);
  });
  log.info("theme toggle: wired");
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
