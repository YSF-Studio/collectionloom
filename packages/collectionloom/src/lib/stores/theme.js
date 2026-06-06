const STORAGE_KEY = "collectionloom-theme";

/** @typedef {"dark" | "light" | "system"} ThemeMode */

const MEDIA = typeof window !== "undefined"
  ? window.matchMedia("(prefers-color-scheme: dark)")
  : null;

/** @returns {ThemeMode} */
function readStored() {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    /* ignore */
  }
  return "system";
}

/** @param {ThemeMode} mode */
function resolvedTheme(mode) {
  if (mode === "system") {
    return MEDIA?.matches ? "dark" : "light";
  }
  return mode === "dark" ? "dark" : "light";
}

/** @param {"dark" | "light"} resolved */
function applyResolved(resolved) {
  const root = document.documentElement;
  const body = document.body;
  if (resolved === "light") {
    root.classList.add("light-mode");
    body.classList.add("light-mode");
  } else {
    root.classList.remove("light-mode");
    body.classList.remove("light-mode");
  }
}

/** @param {ThemeMode} mode */
export function applyTheme(mode) {
  applyResolved(resolvedTheme(mode));
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* ignore */
  }
}

let mediaCleanup = null;

export function initTheme() {
  const mode = readStored();
  applyTheme(mode);
  if (MEDIA) {
    const onChange = () => {
      if (readStored() === "system") applyResolved(resolvedTheme("system"));
    };
    MEDIA.addEventListener("change", onChange);
    mediaCleanup = () => MEDIA.removeEventListener("change", onChange);
  }
  return mediaCleanup;
}

/** @returns {ThemeMode} */
export function getTheme() {
  return readStored();
}

/** @returns {"dark" | "light"} */
export function getResolvedTheme() {
  return resolvedTheme(readStored());
}

/** @param {ThemeMode} mode */
export function setTheme(mode) {
  applyTheme(mode);
}

/** Cycle light → dark → system and return new mode */
export function cycleTheme() {
  const order = /** @type {ThemeMode[]} */ (["light", "dark", "system"]);
  const current = readStored();
  const idx = order.indexOf(current);
  const next = order[(idx + 1) % order.length];
  applyTheme(next);
  return next;
}

/** @deprecated use cycleTheme */
export function toggleTheme() {
  return cycleTheme();
}
