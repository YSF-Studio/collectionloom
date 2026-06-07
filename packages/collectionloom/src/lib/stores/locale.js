const STORAGE_KEY = "collectionloom-locale";
const listeners = new Set();

/** @typedef {"en" | "id" | "system"} LocaleMode */

function systemLocale() {
  if (typeof navigator === "undefined") return "en";
  const lang = (navigator.language || navigator.languages?.[0] || "en").toLowerCase();
  return lang.startsWith("id") ? "id" : "en";
}

/** @returns {LocaleMode} */
function readStored() {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "en" || v === "id" || v === "system") return v;
  } catch {
    /* ignore */
  }
  return "system";
}

/** @param {LocaleMode} mode */
function resolvedLocale(mode) {
  return mode === "system" ? systemLocale() : mode;
}

/** @param {LocaleMode} mode */
function applyLocale(mode) {
  const resolved = resolvedLocale(mode);
  const root = document.documentElement;
  root.lang = resolved;
  root.setAttribute("lang", resolved);
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* ignore */
  }
  for (const listener of listeners) {
    try {
      listener(mode, resolved);
    } catch {
      /* ignore listener errors */
    }
  }
}

/** @param {LocaleMode} mode */
export function setLocale(mode) {
  applyLocale(mode);
}

export function getLocale() {
  return readStored();
}

export function getResolvedLocale() {
  return resolvedLocale(readStored());
}

export function cycleLocale() {
  const order = /** @type {LocaleMode[]} */ (["en", "id", "system"]);
  const current = readStored();
  const idx = order.indexOf(current);
  const next = order[(idx + 1) % order.length];
  applyLocale(next);
  return next;
}

export function initLocale() {
  applyLocale(readStored());
}

/** @param {(mode: LocaleMode, resolved: "en" | "id") => void} listener */
export function subscribeLocale(listener) {
  listeners.add(listener);
  try {
    listener(readStored(), resolvedLocale(readStored()));
  } catch {
    /* ignore */
  }
  return () => {
    listeners.delete(listener);
  };
}
